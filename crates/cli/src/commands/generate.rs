use std::fs;
use std::path::{Path, PathBuf};

use boundra_core::load_project_model;
use serde_json::Value;

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
            eprintln!("failed to load project: {err}");
            return 2;
        }
    };

    // Boundra 방식에서는 도메인을 먼저 만든 뒤 그 안에 route/query/mutation을 생성한다.
    // 그래서 존재하지 않는 도메인에는 파일을 만들지 않는다.
    if !project.domains.contains_key(&options.domain) {
        eprintln!("unknown domain: {}", options.domain);
        return 2;
    }

    let domain_root = options
        .root
        .join(&project.config.paths.domains)
        .join(&options.domain);
    let created = match scaffold_generated_artifact(&domain_root, options) {
        Ok(created) => created,
        Err(err) => {
            eprintln!("failed to generate artifact: {err}");
            return 3;
        }
    };
    let public_api_path = format!("./shared/contracts/{}.ts", options.name);
    if let Err(err) = update_domain_manifest_public_api(
        &domain_root.join(&project.config.domain.manifest_file),
        &public_api_path,
    ) {
        eprintln!("failed to update domain manifest: {err}");
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

            write_new_file(
                &contract_path,
                &format!(
                    "import type {{ BoundraRoute }} from '@boundra/runtime';\n\nexport type {type_name}Input = Record<string, never>;\nexport type {type_name}Result = Record<string, never>;\n\nexport const {function_name}Route: BoundraRoute<{type_name}Input, {type_name}Result> = {{\n  kind: 'route',\n  name: '{name}',\n}};\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            write_new_file(
                &route_path,
                &format!(
                    "import type {{ {type_name}Input, {type_name}Result }} from '../../shared/contracts/{name}';\n\nexport async function {function_name}(input: {type_name}Input): Promise<{type_name}Result> {{\n  void input;\n  return {{}};\n}}\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            created.push(contract_path);
            created.push(route_path);
        }
        GenerateKind::Query => {
            // query는 client hook과 shared query contract를 함께 만든다.
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let query_path = domain_root
                .join("client")
                .join("queries")
                .join(format!("use-{}.ts", options.name));

            write_new_file(
                &contract_path,
                &format!(
                    "import type {{ BoundraQuery }} from '@boundra/runtime';\n\nexport type {type_name}QueryInput = Record<string, never>;\nexport type {type_name}QueryResult = Record<string, never>;\n\nexport const {function_name}Query: BoundraQuery<{type_name}QueryInput, {type_name}QueryResult> = {{\n  kind: 'query',\n  name: '{name}',\n}};\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            write_new_file(
                &query_path,
                &format!(
                    "import type {{ {type_name}QueryInput, {type_name}QueryResult }} from '../../shared/contracts/{name}';\n\nexport function use{type_name}Query(input: {type_name}QueryInput): {type_name}QueryResult {{\n  void input;\n  return {{}};\n}}\n",
                    name = options.name
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
                .join(format!("use-{}.ts", options.name));

            write_new_file(
                &contract_path,
                &format!(
                    "import type {{ BoundraMutation }} from '@boundra/runtime';\n\nexport type {type_name}MutationInput = Record<string, never>;\nexport type {type_name}MutationResult = Record<string, never>;\n\nexport const {function_name}Mutation: BoundraMutation<{type_name}MutationInput, {type_name}MutationResult> = {{\n  kind: 'mutation',\n  name: '{name}',\n}};\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            write_new_file(
                &mutation_path,
                &format!(
                    "import type {{ {type_name}MutationInput, {type_name}MutationResult }} from '../../shared/contracts/{name}';\n\nexport function use{type_name}Mutation(input: {type_name}MutationInput): {type_name}MutationResult {{\n  void input;\n  return {{}};\n}}\n",
                    name = options.name
                ),
            )?;
            created.push(contract_path);
            created.push(mutation_path);
        }
    }

    Ok(created)
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
