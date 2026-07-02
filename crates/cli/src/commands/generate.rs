use std::fs;
use std::path::{Path, PathBuf};

use boundra_core::load_project_model;
use serde_json::Value;

use crate::output::{print_error, CliDiagnostic};
use crate::util::{camel_case, display_path, pascal_case};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GenerateOptions {
    pub(crate) kind: GenerateKind,
    pub(crate) domain: String,
    pub(crate) name: String,
    pub(crate) root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenerateKind {
    Route,
    Query,
    Mutation,
}

pub(crate) fn run(options: &GenerateOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            print_error(
                &CliDiagnostic::new(
                    "PROJECT-001",
                    format!("failed to load project: {err}"),
                    "fix the reported config or domain manifest and retry",
                )
                .with_context("root", options.root.display().to_string()),
            );
            return 2;
        }
    };

    // Boundra 방식에서는 도메인을 먼저 만든 뒤 그 안에 route/query/mutation을 생성한다.
    // 그래서 존재하지 않는 도메인에는 파일을 만들지 않는다.
    if !project.domains.contains_key(&options.domain) {
        let available = project
            .domains
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        print_error(
            &CliDiagnostic::new(
                "DOMAIN-004",
                format!("unknown domain '{}'", options.domain),
                format!(
                    "run 'boundra create-domain {}' or choose an existing domain",
                    options.domain
                ),
            )
            .with_context("available", available)
            .with_context("domain", &options.domain),
        );
        return 2;
    }

    let domain_root = options
        .root
        .join(&project.config.paths.domains)
        .join(&options.domain);
    let created = match scaffold_generated_artifact(&domain_root, options) {
        Ok(created) => created,
        Err(err) => {
            let is_conflict = err.kind() == std::io::ErrorKind::AlreadyExists;
            print_error(
                &CliDiagnostic::new(
                    "GEN-001",
                    format!("failed to generate artifact: {err}"),
                    if is_conflict {
                        "choose a new resource name; Boundra never overwrites generated files"
                    } else {
                        "check workspace permissions and retry"
                    },
                )
                .with_context("resource", format!("{}/{}", options.domain, options.name)),
            );
            return if is_conflict { 2 } else { 3 };
        }
    };
    let public_api_path = format!("./shared/contracts/{}.ts", options.name);
    if let Err(err) = update_domain_manifest_public_api(
        &domain_root.join(&project.config.domain.manifest_file),
        &public_api_path,
    ) {
        print_error(
            &CliDiagnostic::new(
                "GEN-002",
                format!("failed to update domain manifest: {err}"),
                "fix the domain manifest and register the generated contract before retrying",
            )
            .with_context("domain", &options.domain),
        );
        return 3;
    }
    if let Err(err) = update_shared_public_api(&domain_root, &options.name) {
        print_error(
            &CliDiagnostic::new(
                "GEN-003",
                format!("failed to update shared public API: {err}"),
                "check shared/public.ts permissions and export the generated contract",
            )
            .with_context("domain", &options.domain),
        );
        return 3;
    }

    println!(
        "generate {}: OK ({}/{})",
        generate_kind_name(options.kind),
        options.domain,
        options.name
    );
    for path in created {
        println!("created: {}", display_path(&path));
    }
    0
}

fn update_domain_manifest_public_api(
    manifest_path: &Path,
    public_api_path: &str,
) -> std::io::Result<()> {
    if !manifest_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(manifest_path)?;
    let mut manifest: Value = serde_json::from_str(&content).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid JSON in {}: {err}", display_path(manifest_path)),
        )
    })?;

    let Some(manifest_object) = manifest.as_object_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "domain manifest must be a JSON object",
        ));
    };

    let public_api = manifest_object
        .entry("publicApi")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(public_api_object) = public_api.as_object_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "domain manifest publicApi must be a JSON object",
        ));
    };

    let shared = public_api_object
        .entry("shared")
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(shared_array) = shared.as_array_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "domain manifest publicApi.shared must be an array",
        ));
    };

    if !shared_array
        .iter()
        .any(|value| value.as_str() == Some(public_api_path))
    {
        shared_array.push(Value::String(public_api_path.to_string()));
    }

    let output =
        serde_json::to_string_pretty(&manifest).expect("failed to serialize domain manifest");
    fs::write(manifest_path, format!("{output}\n"))
}

fn scaffold_generated_artifact(
    domain_root: &Path,
    options: &GenerateOptions,
) -> std::io::Result<Vec<PathBuf>> {
    // 파일명은 kebab-case를 유지하고, TypeScript 타입/함수명만 PascalCase/camelCase로 바꾼다.
    // 예: create-invoice -> CreateInvoice 타입, createInvoice 함수
    let type_name = pascal_case(&options.name);
    let mut created = Vec::new();

    match options.kind {
        GenerateKind::Route => {
            // route는 server entry와 shared contract를 함께 만든다.
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let route_path = domain_root
                .join("server")
                .join("routes")
                .join(format!("{}.ts", options.name));

            ensure_new_files([contract_path.as_path(), route_path.as_path()])?;

            write_new_file(
                &contract_path,
                &format!(
                    "import {{ defineRoute, type InferSchema }} from \"boundra\";\nimport {{ z }} from \"zod\";\n\nexport const {function_name}InputSchema = z.object({{}});\nexport const {function_name}ResultSchema = z.object({{}});\n\nexport type {type_name}Input = InferSchema<typeof {function_name}InputSchema>;\nexport type {type_name}Result = InferSchema<typeof {function_name}ResultSchema>;\n\nexport const {function_name}Route = defineRoute({{\n  name: \"{name}\",\n  input: {function_name}InputSchema,\n  result: {function_name}ResultSchema,\n}});\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            write_new_file(
                &route_path,
                &format!(
                    "import {{ implementRoute }} from \"boundra\";\n\nimport {{ {function_name}Route }} from \"../../shared/contracts/{name}\";\n\nexport const {function_name} = implementRoute(\n  {function_name}Route,\n  async (input) => {{\n    void input;\n    return {{}};\n  }},\n);\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            created.push(contract_path);
            created.push(route_path);
        }
        GenerateKind::Query => {
            // query는 framework-neutral client adapter와 shared contract를 함께 만든다.
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let query_path = domain_root
                .join("client")
                .join("queries")
                .join(format!("{}.ts", options.name));

            ensure_new_files([contract_path.as_path(), query_path.as_path()])?;

            write_new_file(
                &contract_path,
                &format!(
                    "import {{ defineQuery, type InferSchema }} from \"boundra\";\nimport {{ z }} from \"zod\";\n\nexport const {function_name}InputSchema = z.object({{}});\nexport const {function_name}ResultSchema = z.object({{}});\n\nexport type {type_name}QueryInput = InferSchema<typeof {function_name}InputSchema>;\nexport type {type_name}QueryResult = InferSchema<typeof {function_name}ResultSchema>;\n\nexport const {function_name}Query = defineQuery({{\n  name: \"{name}\",\n  input: {function_name}InputSchema,\n  result: {function_name}ResultSchema,\n}});\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            write_new_file(
                &query_path,
                &format!(
                    "import type {{ BoundraClient }} from \"boundra\";\n\nimport {{\n  {function_name}Query,\n  type {type_name}QueryInput,\n}} from \"../../shared/contracts/{name}\";\n\nexport function {function_name}(\n  client: BoundraClient,\n  input: {type_name}QueryInput,\n) {{\n  return client.query({function_name}Query, input);\n}}\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            created.push(contract_path);
            created.push(query_path);
        }
        GenerateKind::Mutation => {
            // mutation도 query와 같은 구조지만 쓰기 작업임을 이름으로 구분한다.
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let mutation_path = domain_root
                .join("client")
                .join("mutations")
                .join(format!("{}.ts", options.name));

            ensure_new_files([contract_path.as_path(), mutation_path.as_path()])?;

            write_new_file(
                &contract_path,
                &format!(
                    "import {{ defineMutation, type InferSchema }} from \"boundra\";\nimport {{ z }} from \"zod\";\n\nexport const {function_name}InputSchema = z.object({{}});\nexport const {function_name}ResultSchema = z.object({{}});\n\nexport type {type_name}MutationInput = InferSchema<typeof {function_name}InputSchema>;\nexport type {type_name}MutationResult = InferSchema<typeof {function_name}ResultSchema>;\n\nexport const {function_name}Mutation = defineMutation({{\n  name: \"{name}\",\n  input: {function_name}InputSchema,\n  result: {function_name}ResultSchema,\n}});\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            write_new_file(
                &mutation_path,
                &format!(
                    "import type {{ BoundraClient }} from \"boundra\";\n\nimport {{\n  {function_name}Mutation,\n  type {type_name}MutationInput,\n}} from \"../../shared/contracts/{name}\";\n\nexport function {function_name}(\n  client: BoundraClient,\n  input: {type_name}MutationInput,\n) {{\n  return client.mutation({function_name}Mutation, input);\n}}\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            created.push(contract_path);
            created.push(mutation_path);
        }
    }

    Ok(created)
}

fn ensure_new_files<'a>(paths: impl IntoIterator<Item = &'a Path>) -> std::io::Result<()> {
    for path in paths {
        if path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("file already exists: {}", display_path(path)),
            ));
        }
    }
    Ok(())
}

fn update_shared_public_api(domain_root: &Path, name: &str) -> std::io::Result<()> {
    let public_path = domain_root.join("shared").join("public.ts");
    let export_line = format!("export * from \"./contracts/{name}\";\n");
    let existing = if public_path.exists() {
        fs::read_to_string(&public_path)?
    } else {
        String::new()
    };

    if existing.lines().any(|line| line == export_line.trim_end()) {
        return Ok(());
    }

    let output = if existing.trim() == "export {};" || existing.trim().is_empty() {
        export_line
    } else {
        format!(
            "{}{separator}{export_line}",
            existing.trim_end(),
            separator = "\n"
        )
    };
    fs::write(public_path, output)
}

fn write_new_file(path: &Path, content: &str) -> std::io::Result<()> {
    // 생성기가 기존 파일을 덮어쓰면 사용자 코드를 잃을 수 있으므로 항상 실패시킨다.
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("file already exists: {}", display_path(path)),
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

pub(crate) fn generate_kind_name(kind: GenerateKind) -> &'static str {
    match kind {
        GenerateKind::Route => "route",
        GenerateKind::Query => "query",
        GenerateKind::Mutation => "mutation",
    }
}
