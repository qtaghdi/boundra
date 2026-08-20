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
    Resource,
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
    let mut snapshots = Vec::new();
    let manifest_path = domain_root.join(&project.config.domain.manifest_file);
    let public_api_path = format!("./shared/contracts/{}.ts", options.name);
    if let Err(err) = snapshot_then_update(&mut snapshots, &manifest_path, || {
        update_domain_manifest_public_api(&manifest_path, &public_api_path)
    }) {
        report_generation_update_error(
            "GEN-002",
            "domain manifest",
            &err,
            "fix the domain manifest and register the generated contract before retrying",
            options,
            &manifest_path,
            rollback_generation(&created, &snapshots),
        );
        return 3;
    }
    let shared_public_path = domain_root.join("shared").join("public.ts");
    if let Err(err) = snapshot_then_update(&mut snapshots, &shared_public_path, || {
        update_shared_public_api(&domain_root, &options.name)
    }) {
        report_generation_update_error(
            "GEN-003",
            "shared public API",
            &err,
            "check shared/public.ts permissions and export the generated contract",
            options,
            &shared_public_path,
            rollback_generation(&created, &snapshots),
        );
        return 3;
    }
    if let Some(client_export) = generated_client_export(options.kind, &options.name) {
        let client_public_path = domain_root.join("client").join("public.ts");
        if let Err(err) = snapshot_then_update(&mut snapshots, &client_public_path, || {
            update_client_public_api(&domain_root, &client_export)
        }) {
            report_generation_update_error(
                "GEN-004",
                "client public API",
                &err,
                "check client/public.ts permissions and export the generated adapter",
                options,
                &client_public_path,
                rollback_generation(&created, &snapshots),
            );
            return 3;
        }
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
                    "import type {{ BoundraCallOptions, BoundraClient }} from \"boundra\";\n\nimport {{\n  {function_name}Query,\n  type {type_name}QueryInput,\n}} from \"../../shared/contracts/{name}\";\n\nexport function {function_name}(\n  client: BoundraClient,\n  input: {type_name}QueryInput,\n  options?: BoundraCallOptions,\n) {{\n  return client.query({function_name}Query, input, options);\n}}\n",
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
                    "import type {{ BoundraCallOptions, BoundraClient }} from \"boundra\";\n\nimport {{\n  {function_name}Mutation,\n  type {type_name}MutationInput,\n}} from \"../../shared/contracts/{name}\";\n\nexport function {function_name}(\n  client: BoundraClient,\n  input: {type_name}MutationInput,\n  options?: BoundraCallOptions,\n) {{\n  return client.mutation({function_name}Mutation, input, options);\n}}\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            created.push(contract_path);
            created.push(mutation_path);
        }
        GenerateKind::Resource => {
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let resource_path = domain_root
                .join("client")
                .join("resources")
                .join(format!("{}.ts", options.name));
            ensure_new_files([contract_path.as_path(), resource_path.as_path()])?;

            let name = &options.name;
            let function_name = camel_case(name);
            let plural_name = resource_plural_name(name);
            write_new_file(
                &contract_path,
                &format!(
                    "import {{ defineMutation, defineQuery, type InferSchema }} from \"boundra\";\nimport {{ z }} from \"zod\";\n\nexport const {function_name}FieldsSchema = z.object({{}});\nexport const {function_name}Schema = {function_name}FieldsSchema.extend({{ id: z.string().min(1) }});\nexport type {type_name} = InferSchema<typeof {function_name}Schema>;\n\nexport const list{plural_type_name}Query = defineQuery({{\n  name: \"list-{plural_name}\",\n  input: z.object({{}}),\n  result: z.object({{ items: z.array({function_name}Schema) }}),\n}});\nexport const create{type_name}Mutation = defineMutation({{\n  name: \"create-{name}\",\n  input: {function_name}FieldsSchema,\n  result: z.object({{ item: {function_name}Schema }}),\n}});\nexport const update{type_name}Mutation = defineMutation({{\n  name: \"update-{name}\",\n  input: {function_name}FieldsSchema.partial().extend({{ id: z.string().min(1) }}),\n  result: z.object({{ item: {function_name}Schema }}),\n}});\nexport const delete{type_name}Mutation = defineMutation({{\n  name: \"delete-{name}\",\n  input: z.object({{ id: z.string().min(1) }}),\n  result: z.object({{ id: z.string().min(1) }}),\n}});\n",
                    type_name = type_name,
                    plural_type_name = pascal_case(&plural_name),
                ),
            )?;
            write_new_file(
                &resource_path,
                &format!(
                    "import type {{ BoundraCallOptions, BoundraClient, ContractInput }} from \"boundra\";\n\nimport {{\n  create{type_name}Mutation,\n  delete{type_name}Mutation,\n  list{plural_type_name}Query,\n  update{type_name}Mutation,\n}} from \"../../shared/contracts/{name}\";\n\nexport const list{plural_type_name} = (client: BoundraClient, options?: BoundraCallOptions) => client.query(list{plural_type_name}Query, {{}}, options);\nexport const create{type_name} = (client: BoundraClient, input: ContractInput<typeof create{type_name}Mutation>, options?: BoundraCallOptions) => client.mutation(create{type_name}Mutation, input, options);\nexport const update{type_name} = (client: BoundraClient, input: ContractInput<typeof update{type_name}Mutation>, options?: BoundraCallOptions) => client.mutation(update{type_name}Mutation, input, options);\nexport const delete{type_name} = (client: BoundraClient, input: ContractInput<typeof delete{type_name}Mutation>, options?: BoundraCallOptions) => client.mutation(delete{type_name}Mutation, input, options);\n",
                    plural_type_name = pascal_case(&plural_name),
                ),
            )?;
            created.push(contract_path);
            created.push(resource_path);
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
    append_idempotent_export(&public_path, &format!("./contracts/{name}"))
}

/// Export one generated client adapter through the stable client barrel.
fn update_client_public_api(domain_root: &Path, export_path: &str) -> std::io::Result<()> {
    let public_path = domain_root.join("client").join("public.ts");
    append_idempotent_export(&public_path, export_path)
}

/// Append an export unless the same module path is already exported.
fn append_idempotent_export(public_path: &Path, export_path: &str) -> std::io::Result<()> {
    let existing = if public_path.exists() {
        fs::read_to_string(public_path)?
    } else {
        String::new()
    };

    if existing
        .lines()
        .any(|line| exported_module_path(line) == Some(export_path))
    {
        return Ok(());
    }

    let export_line = format!("export * from \"{export_path}\";\n");
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

/// Extract the module path from a single `export * from` declaration.
fn exported_module_path(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("export")?.trim_start();
    let rest = rest.strip_prefix('*')?.trim_start();
    let rest = rest.strip_prefix("from")?.trim_start();
    let quote = rest.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let quoted = &rest[quote.len_utf8()..];
    let end = quoted.find(quote)?;
    Some(&quoted[..end])
}

#[derive(Debug)]
struct FileSnapshot {
    path: PathBuf,
    contents: Option<Vec<u8>>,
}

/// Snapshot a file immediately before applying one generation update.
fn snapshot_then_update(
    snapshots: &mut Vec<FileSnapshot>,
    path: &Path,
    update: impl FnOnce() -> std::io::Result<()>,
) -> std::io::Result<()> {
    let contents = if path.exists() {
        Some(fs::read(path)?)
    } else {
        None
    };
    snapshots.push(FileSnapshot {
        path: path.to_path_buf(),
        contents,
    });
    update()
}

/// Restore updated files and remove artifacts created by the failed command.
fn rollback_generation(created: &[PathBuf], snapshots: &[FileSnapshot]) -> std::io::Result<()> {
    let mut first_error = None;

    for snapshot in snapshots.iter().rev() {
        let result = match &snapshot.contents {
            Some(contents) => fs::write(&snapshot.path, contents),
            None if snapshot.path.exists() => fs::remove_file(&snapshot.path),
            None => Ok(()),
        };
        if first_error.is_none() {
            first_error = result.err();
        }
    }
    for path in created.iter().rev() {
        if path.exists() {
            if let Err(err) = fs::remove_file(path) {
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
        }
    }

    first_error.map_or(Ok(()), Err)
}

/// Emit a stage-specific generation error and surface rollback failures.
fn report_generation_update_error(
    code: &str,
    target: &str,
    err: &std::io::Error,
    suggestion: &str,
    options: &GenerateOptions,
    file: &Path,
    rollback: std::io::Result<()>,
) {
    let mut diagnostic = CliDiagnostic::new(
        code,
        format!("failed to update {target}: {err}"),
        suggestion,
    )
    .with_context("domain", &options.domain)
    .with_context("file", display_path(file));
    if let Err(rollback_error) = rollback {
        diagnostic = diagnostic.with_context("rollback_error", rollback_error.to_string());
    }
    print_error(&diagnostic);
}

/// Map client-producing generators to their public barrel path.
fn generated_client_export(kind: GenerateKind, name: &str) -> Option<String> {
    let directory = match kind {
        GenerateKind::Query => "queries",
        GenerateKind::Mutation => "mutations",
        GenerateKind::Resource => "resources",
        GenerateKind::Route => return None,
    };
    Some(format!("./{directory}/{name}"))
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
        GenerateKind::Resource => "resource",
    }
}

fn resource_plural_name(name: &str) -> String {
    if name.ends_with('s') {
        name.to_string()
    } else {
        format!("{name}s")
    }
}
