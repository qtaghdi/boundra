use std::collections::BTreeMap;

use boundra_core::{
    CheckBoundariesConfig, DomainManifest, Layer, PathAlias, PublicApi, RuleCode, Violation,
};
use boundra_parser::ImportRecord;

pub fn check_boundaries(imports: &[ImportRecord]) -> Vec<Violation> {
    check_boundaries_with_context(imports, &BoundaryContext::default())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryContext {
    pub apps_path: String,
    pub domains_path: String,
    pub packages_path: String,
    pub domains: BTreeMap<String, DomainManifest>,
    pub path_aliases: Vec<PathAlias>,
}

impl Default for BoundaryContext {
    fn default() -> Self {
        Self {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::new(),
            path_aliases: Vec::new(),
        }
    }
}

pub fn check_boundaries_with_context(
    imports: &[ImportRecord],
    context: &BoundaryContext,
) -> Vec<Violation> {
    check_boundaries_with_config(imports, context, &CheckBoundariesConfig::default())
}

pub fn check_boundaries_with_config(
    imports: &[ImportRecord],
    context: &BoundaryContext,
    config: &CheckBoundariesConfig,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for record in imports {
        let (source_domain, source_layer) =
            parse_domain_path_with_context(&record.source_file, context)
                .unwrap_or((String::new(), Layer::Unknown));

        let resolved_target =
            resolve_import_path_with_context(&record.source_dir, &record.import_path, context);

        if source_layer == Layer::Shared
            && has_denied_shared_capability(
                &record.import_path,
                resolved_target.as_deref(),
                context,
                config,
            )
        {
            violations.push(Violation {
                rule: RuleCode::Br003,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: "shared layer cannot depend on a capability denied by boundary policy"
                    .to_string(),
                suggestion:
                    "move the dependency to client/server or adjust the shared capability policy"
                        .to_string(),
            });
        }

        let Some(target) = resolved_target else {
            continue;
        };
        let (target_domain, target_layer) = parse_domain_path_with_context(&target, context)
            .unwrap_or((String::new(), Layer::Unknown));

        match (source_layer, target_layer) {
            (Layer::Client, Layer::Server) => violations.push(Violation {
                rule: RuleCode::Br001,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: "client layer cannot import server layer".to_string(),
                suggestion: "move shared contract to shared layer or call through an API boundary"
                    .to_string(),
            }),
            (Layer::Server, Layer::Client) => violations.push(Violation {
                rule: RuleCode::Br002,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: "server layer cannot import client layer".to_string(),
                suggestion: "move reusable logic to shared layer and avoid reverse dependency"
                    .to_string(),
            }),
            _ => {}
        }

        if is_cross_domain_internal_import(&source_domain, &target_domain, &target, context) {
            violations.push(Violation {
                rule: RuleCode::Br004,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: "domains cannot import another domain's internal path".to_string(),
                suggestion: "import from the target domain's public API instead".to_string(),
            });
        }

        if is_undeclared_cross_domain_public_import(
            &source_domain,
            &target_domain,
            &target,
            context,
        ) {
            violations.push(Violation {
                rule: RuleCode::Br006,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: format!(
                    "domain '{source_domain}' imports undeclared dependency '{target_domain}'"
                ),
                suggestion: format!(
                    "run 'boundra add-dependency {source_domain}/{target_domain}' or remove the import"
                ),
            });
        }

        if is_app_internal_import(
            &record.source_file,
            &source_domain,
            &target_domain,
            &target,
            context,
        ) {
            violations.push(Violation {
                rule: RuleCode::Br005,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: "apps cannot import a domain's internal path".to_string(),
                suggestion: "import from the target domain's declared public API instead"
                    .to_string(),
            });
        }
    }

    violations
}

pub fn resolve_import_path(source_dir: &str, import_path: &str) -> Option<String> {
    resolve_import_path_with_context(source_dir, import_path, &BoundaryContext::default())
}

pub fn resolve_import_path_with_context(
    source_dir: &str,
    import_path: &str,
    context: &BoundaryContext,
) -> Option<String> {
    // 이미 workspace 기준 경로면 그대로 정규화한다.
    if is_within_path(import_path, &context.domains_path) {
        return Some(normalize_path(import_path));
    }
    // 외부 패키지는 None으로 두지만, tsconfig alias는 내부 경로일 수 있으므로 먼저 풀어본다.
    if !import_path.starts_with('.') {
        return resolve_aliased_import_path(import_path, &context.path_aliases);
    }

    // 상대 import는 현재 파일의 디렉터리를 기준으로 workspace 상대 경로로 바꾼다.
    let joined = if source_dir.is_empty() {
        import_path.to_string()
    } else {
        format!("{source_dir}/{import_path}")
    };

    Some(normalize_path(&joined))
}

fn resolve_aliased_import_path(import_path: &str, path_aliases: &[PathAlias]) -> Option<String> {
    // alias는 prefix 치환만 한다. 실제 파일 존재 여부는 boundary 판단에 필요하지 않다.
    for alias in path_aliases {
        if let Some(rest) = import_path.strip_prefix(&alias.prefix) {
            return Some(normalize_path(&format!("{}{}", alias.target_prefix, rest)));
        }
    }

    None
}

fn parse_domain_path(path: &str, domains_path: &str) -> Option<(String, Layer)> {
    let normalized = normalize_path(path);
    let normalized_root = normalize_path(domains_path);
    let relative = if normalized_root.is_empty() {
        normalized.as_str()
    } else {
        normalized.strip_prefix(&format!("{normalized_root}/"))?
    };
    let mut parts = relative.split('/');

    let domain = parts.next()?.to_string();
    let layer = match parts.next()? {
        "client" => Layer::Client,
        "server" => Layer::Server,
        "shared" => Layer::Shared,
        "mcp" => Layer::Mcp,
        "tests" => Layer::Tests,
        _ => Layer::Unknown,
    };

    Some((domain, layer))
}

fn parse_domain_path_with_context(
    path: &str,
    context: &BoundaryContext,
) -> Option<(String, Layer)> {
    let (domain, layer) = parse_domain_path(path, &context.domains_path)?;
    if layer != Layer::Unknown || !is_direct_domain_child(path, &context.domains_path) {
        return Some((domain, layer));
    }

    let compact_layer = context
        .domains
        .get(&domain)
        .and_then(|manifest| single_public_api_layer(&manifest.public_api))
        .unwrap_or(Layer::Unknown);
    Some((domain, compact_layer))
}

fn is_direct_domain_child(path: &str, domains_path: &str) -> bool {
    let normalized = normalize_path(path);
    let normalized_root = normalize_path(domains_path);
    let relative = if normalized_root.is_empty() {
        normalized.as_str()
    } else if let Some(relative) = normalized.strip_prefix(&format!("{normalized_root}/")) {
        relative
    } else {
        return false;
    };

    relative.split('/').count() == 2
}

fn single_public_api_layer(public_api: &PublicApi) -> Option<Layer> {
    let layers = [
        (!public_api.client.is_empty(), Layer::Client),
        (!public_api.server.is_empty(), Layer::Server),
        (!public_api.shared.is_empty(), Layer::Shared),
    ];
    let mut declared = layers
        .into_iter()
        .filter_map(|(is_declared, layer)| is_declared.then_some(layer));
    let layer = declared.next()?;
    declared.next().is_none().then_some(layer)
}

fn is_cross_domain_internal_import(
    source_domain: &str,
    target_domain: &str,
    target_path: &str,
    context: &BoundaryContext,
) -> bool {
    if source_domain.is_empty() || target_domain.is_empty() || source_domain == target_domain {
        return false;
    }

    !is_public_api_path(target_domain, target_path, context)
}

fn is_undeclared_cross_domain_public_import(
    source_domain: &str,
    target_domain: &str,
    target_path: &str,
    context: &BoundaryContext,
) -> bool {
    if source_domain.is_empty() || target_domain.is_empty() || source_domain == target_domain {
        return false;
    }
    if !is_public_api_path(target_domain, target_path, context) {
        return false;
    }

    context.domains.get(source_domain).is_some_and(|manifest| {
        !manifest
            .depends_on
            .iter()
            .any(|dependency| dependency == target_domain)
    })
}

fn is_app_internal_import(
    source_file: &str,
    source_domain: &str,
    target_domain: &str,
    target_path: &str,
    context: &BoundaryContext,
) -> bool {
    if !source_domain.is_empty()
        || target_domain.is_empty()
        || !is_within_path(source_file, &context.apps_path)
    {
        return false;
    }

    !is_public_api_path(target_domain, target_path, context)
}

fn is_within_path(path: &str, root: &str) -> bool {
    let normalized_path = normalize_path(path);
    let normalized_root = normalize_path(root).trim_end_matches('/').to_string();

    if normalized_root.is_empty() {
        return true;
    }

    normalized_path == normalized_root
        || normalized_path.starts_with(&format!("{normalized_root}/"))
}

fn has_denied_shared_capability(
    import_path: &str,
    resolved_target: Option<&str>,
    context: &BoundaryContext,
    config: &CheckBoundariesConfig,
) -> bool {
    let denied = &config.policy.shared.deny_capabilities;
    if denied.is_empty() {
        return false;
    }

    if config
        .capabilities
        .external
        .iter()
        .any(|(source, capabilities)| {
            matches_external_capability_source(import_path, source)
                && contains_denied_capability(capabilities, denied)
        })
    {
        return true;
    }

    let Some(target) = resolved_target else {
        return false;
    };

    if let Some(package) = workspace_package_name(target, &context.packages_path) {
        if config
            .capabilities
            .packages
            .get(&package)
            .is_some_and(|capabilities| contains_denied_capability(capabilities, denied))
        {
            return true;
        }
    }

    is_workspace_app_path(target, context)
        && contains_denied_capability(&config.capabilities.apps, denied)
}

fn contains_denied_capability(capabilities: &[String], denied: &[String]) -> bool {
    capabilities
        .iter()
        .any(|capability| denied.iter().any(|blocked| blocked == capability))
}

fn matches_external_capability_source(import_path: &str, source: &str) -> bool {
    let normalized_import = import_path.replace('\\', "/");
    if let Some(prefix) = source.strip_suffix('*') {
        return normalized_import.starts_with(prefix);
    }
    normalized_import == source || normalized_import.starts_with(&format!("{source}/"))
}

fn workspace_package_name(path: &str, packages_path: &str) -> Option<String> {
    let normalized_path = normalize_path(path);
    let normalized_root = normalize_path(packages_path)
        .trim_end_matches('/')
        .to_string();
    let relative = if normalized_root.is_empty() {
        normalized_path.as_str()
    } else {
        normalized_path.strip_prefix(&format!("{normalized_root}/"))?
    };
    let package = relative.split('/').next()?;
    (!package.is_empty()).then(|| package.to_string())
}

fn is_workspace_app_path(path: &str, context: &BoundaryContext) -> bool {
    if normalize_path(&context.apps_path).is_empty() {
        return !is_within_path(path, &context.domains_path)
            && !is_within_path(path, &context.packages_path);
    }

    is_within_path(path, &context.apps_path)
}

fn is_public_api_path(domain: &str, target_path: &str, context: &BoundaryContext) -> bool {
    let normalized_path = normalize_path(target_path);
    let normalized = strip_ts_like_extension(&normalized_path);

    if let Some(manifest) = context.domains.get(domain) {
        return manifest.public_api.all_paths().any(|public_path| {
            normalized == normalize_public_api_path(domain, public_path, &context.domains_path)
        });
    }

    normalized == normalize_path(&format!("{}/{domain}/shared/public", context.domains_path))
}

fn normalize_public_api_path(domain: &str, public_path: &str, domains_path: &str) -> String {
    let relative = public_path.strip_prefix("./").unwrap_or(public_path);
    let normalized = normalize_path(&format!("{domains_path}/{domain}/{relative}"));
    strip_ts_like_extension(&normalized).to_string()
}

fn strip_ts_like_extension(path: &str) -> &str {
    for extension in [".ts", ".tsx", ".js", ".jsx", ".svelte"] {
        if let Some(stripped) = path.strip_suffix(extension) {
            return stripped;
        }
    }

    path
}

fn normalize_path(input: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    let replaced = input.replace('\\', "/");

    for part in replaced.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            x => stack.push(x),
        }
    }

    stack.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundra_core::{CheckBoundariesConfig, DomainManifest, PublicApi};
    use boundra_parser::ImportRecord;

    #[test]
    fn detects_br_001() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/client/use-order.ts".to_string(),
            source_dir: "domains/order/client".to_string(),
            line: 3,
            import_path: "../server/order-service".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br001);
    }

    #[test]
    fn detects_br_002() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/server/order-service.ts".to_string(),
            source_dir: "domains/order/server".to_string(),
            line: 1,
            import_path: "../client/ui".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br002);
    }

    #[test]
    fn detects_br_003_for_shared_ui_dependency() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "react".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn detects_br_003_for_shared_db_dependency() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 2,
            import_path: "@prisma/client".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn detects_br_003_for_shared_workspace_infra_dependency() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 3,
            import_path: "../../../packages/ui/button".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn allows_shared_pure_dependency() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "zod".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert!(violations.is_empty());
    }

    #[test]
    fn configurable_br_003_blocks_custom_external_capability() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "drizzle-orm".to_string(),
        }];
        let mut config = CheckBoundariesConfig::default();
        config
            .capabilities
            .external
            .insert("drizzle-orm".to_string(), vec!["database".to_string()]);

        let violations =
            check_boundaries_with_config(&imports, &BoundaryContext::default(), &config);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn configurable_br_003_blocks_custom_workspace_package_capability() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "../../../packages/persistence/client".to_string(),
        }];
        let mut config = CheckBoundariesConfig::default();
        config
            .capabilities
            .packages
            .insert("persistence".to_string(), vec!["database".to_string()]);

        let violations =
            check_boundaries_with_config(&imports, &BoundaryContext::default(), &config);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn configurable_br_003_can_relax_shared_policy() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "react".to_string(),
        }];
        let mut config = CheckBoundariesConfig::default();
        config.policy.shared.deny_capabilities =
            vec!["database".to_string(), "runtime".to_string()];

        let violations =
            check_boundaries_with_config(&imports, &BoundaryContext::default(), &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn configurable_br_003_can_disable_a_default_source() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "react".to_string(),
        }];
        let mut config = CheckBoundariesConfig::default();
        config
            .capabilities
            .external
            .insert("react".to_string(), Vec::new());

        let violations =
            check_boundaries_with_config(&imports, &BoundaryContext::default(), &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_shared_to_same_domain_shared_import() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/public.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "./schema".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert!(violations.is_empty());
    }

    #[test]
    fn root_app_path_does_not_treat_domain_sources_as_app_runtime() {
        let imports = vec![
            ImportRecord {
                source_file: "domains/auth/shared/contract.ts".to_string(),
                source_dir: "domains/auth/shared".to_string(),
                line: 1,
                import_path: "boundra".to_string(),
            },
            ImportRecord {
                source_file: "domains/auth/shared/contract.ts".to_string(),
                source_dir: "domains/auth/shared".to_string(),
                line: 2,
                import_path: "zod".to_string(),
            },
        ];
        let context = BoundaryContext {
            apps_path: ".".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::new(),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert!(violations.is_empty());
    }

    #[test]
    fn root_app_path_still_blocks_explicit_shared_runtime_dependencies() {
        let imports = vec![ImportRecord {
            source_file: "domains/auth/shared/contract.ts".to_string(),
            source_dir: "domains/auth/shared".to_string(),
            line: 1,
            import_path: "react".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: ".".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::new(),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn detects_br_004_for_cross_domain_internal_import() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/server/checkout.ts".to_string(),
            source_dir: "domains/order/server".to_string(),
            line: 5,
            import_path: "../../product/server/internal/stock".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br004);
    }

    #[test]
    fn allows_cross_domain_shared_public_import() {
        let imports = vec![ImportRecord {
            source_file: "domains/product/client/list.ts".to_string(),
            source_dir: "domains/product/client".to_string(),
            line: 1,
            import_path: "../../auth/shared/public".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_declared_cross_domain_public_api_import() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/server/checkout.ts".to_string(),
            source_dir: "domains/order/server".to_string(),
            line: 1,
            import_path: "../../billing/server/public".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([
                (
                    "billing".to_string(),
                    DomainManifest {
                        name: "billing".to_string(),
                        public_api: PublicApi {
                            client: Vec::new(),
                            server: vec!["./server/public.ts".to_string()],
                            shared: Vec::new(),
                        },
                        depends_on: Vec::new(),
                    },
                ),
                (
                    "order".to_string(),
                    DomainManifest {
                        name: "order".to_string(),
                        public_api: PublicApi {
                            client: Vec::new(),
                            server: Vec::new(),
                            shared: Vec::new(),
                        },
                        depends_on: vec!["billing".to_string()],
                    },
                ),
            ]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_br_006_for_undeclared_cross_domain_public_api_import() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/server/checkout.ts".to_string(),
            source_dir: "domains/order/server".to_string(),
            line: 1,
            import_path: "../../billing/server/public".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([
                (
                    "billing".to_string(),
                    DomainManifest {
                        name: "billing".to_string(),
                        public_api: PublicApi {
                            client: Vec::new(),
                            server: vec!["./server/public.ts".to_string()],
                            shared: Vec::new(),
                        },
                        depends_on: Vec::new(),
                    },
                ),
                (
                    "order".to_string(),
                    DomainManifest {
                        name: "order".to_string(),
                        public_api: PublicApi {
                            client: Vec::new(),
                            server: Vec::new(),
                            shared: Vec::new(),
                        },
                        depends_on: Vec::new(),
                    },
                ),
            ]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br006);
    }

    #[test]
    fn allows_same_domain_internal_import() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/server/checkout.ts".to_string(),
            source_dir: "domains/order/server".to_string(),
            line: 2,
            import_path: "./internal/stock".to_string(),
        }];

        let violations = check_boundaries(&imports);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_alias_resolved_boundary_violation() {
        let imports = vec![ImportRecord {
            source_file: "domains/order/client/use-order.ts".to_string(),
            source_dir: "domains/order/client".to_string(),
            line: 1,
            import_path: "@domains/order/server/checkout".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::new(),
            path_aliases: vec![PathAlias {
                prefix: "@domains/".to_string(),
                target_prefix: "domains/".to_string(),
            }],
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br001);
    }

    #[test]
    fn detects_boundaries_under_a_configured_domain_root() {
        let imports = vec![ImportRecord {
            source_file: "src/lib/domains/order/client/use-order.svelte".to_string(),
            source_dir: "src/lib/domains/order/client".to_string(),
            line: 2,
            import_path: "../server/checkout".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "src/routes".to_string(),
            domains_path: "src/lib/domains".to_string(),
            packages_path: "src/lib/packages".to_string(),
            domains: BTreeMap::new(),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br001);
    }

    #[test]
    fn detects_shared_runtime_packages_under_a_configured_package_root() {
        let imports = vec![ImportRecord {
            source_file: "src/lib/domains/auth/shared/public.ts".to_string(),
            source_dir: "src/lib/domains/auth/shared".to_string(),
            line: 1,
            import_path: "@workspace/ui/button".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "src/routes".to_string(),
            domains_path: "src/lib/domains".to_string(),
            packages_path: "src/lib/packages".to_string(),
            domains: BTreeMap::new(),
            path_aliases: vec![PathAlias {
                prefix: "@workspace/".to_string(),
                target_prefix: "src/lib/packages/".to_string(),
            }],
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn detects_br_005_for_app_to_domain_internal_import() {
        let imports = vec![ImportRecord {
            source_file: "apps/web/src/checkout.ts".to_string(),
            source_dir: "apps/web/src".to_string(),
            line: 4,
            import_path: "../../../domains/order/server/internal/checkout".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([(
                "order".to_string(),
                DomainManifest {
                    name: "order".to_string(),
                    public_api: PublicApi {
                        client: vec!["./client/public.ts".to_string()],
                        server: vec!["./server/public.ts".to_string()],
                        shared: vec!["./shared/public.ts".to_string()],
                    },
                    depends_on: Vec::new(),
                },
            )]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br005);
    }

    #[test]
    fn allows_app_to_domain_declared_public_api_import() {
        let imports = vec![ImportRecord {
            source_file: "frontend/web/src/checkout.ts".to_string(),
            source_dir: "frontend/web/src".to_string(),
            line: 1,
            import_path: "../../../domains/order/client/public".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "frontend".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([(
                "order".to_string(),
                DomainManifest {
                    name: "order".to_string(),
                    public_api: PublicApi {
                        client: vec!["./client/public.ts".to_string()],
                        server: Vec::new(),
                        shared: Vec::new(),
                    },
                    depends_on: Vec::new(),
                },
            )]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_root_app_without_treating_domain_sources_as_apps() {
        let imports = vec![
            ImportRecord {
                source_file: "src/main.ts".to_string(),
                source_dir: "src".to_string(),
                line: 1,
                import_path: "../domains/order/server/internal/checkout".to_string(),
            },
            ImportRecord {
                source_file: "domains/order/server/service.ts".to_string(),
                source_dir: "domains/order/server".to_string(),
                line: 1,
                import_path: "./internal/checkout".to_string(),
            },
        ];
        let context = BoundaryContext {
            apps_path: ".".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([(
                "order".to_string(),
                DomainManifest {
                    name: "order".to_string(),
                    public_api: PublicApi {
                        client: Vec::new(),
                        server: Vec::new(),
                        shared: Vec::new(),
                    },
                    depends_on: Vec::new(),
                },
            )]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br005);
        assert_eq!(violations[0].file, "src/main.ts");
    }

    #[test]
    fn enforces_shared_purity_for_compact_single_layer_domain() {
        let imports = vec![ImportRecord {
            source_file: "domains/appearance-guidance/public.ts".to_string(),
            source_dir: "domains/appearance-guidance".to_string(),
            line: 1,
            import_path: "react".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([(
                "appearance-guidance".to_string(),
                DomainManifest {
                    name: "appearance-guidance".to_string(),
                    public_api: PublicApi {
                        client: Vec::new(),
                        server: Vec::new(),
                        shared: vec!["./public.ts".to_string()],
                    },
                    depends_on: Vec::new(),
                },
            )]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br003);
    }

    #[test]
    fn enforces_client_to_server_rule_for_compact_single_layer_domain() {
        let imports = vec![ImportRecord {
            source_file: "domains/comparison/storage.ts".to_string(),
            source_dir: "domains/comparison".to_string(),
            line: 1,
            import_path: "../analysis/server/public".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([
                (
                    "comparison".to_string(),
                    DomainManifest {
                        name: "comparison".to_string(),
                        public_api: PublicApi {
                            client: vec!["./public.ts".to_string()],
                            server: Vec::new(),
                            shared: Vec::new(),
                        },
                        depends_on: vec!["analysis".to_string()],
                    },
                ),
                (
                    "analysis".to_string(),
                    DomainManifest {
                        name: "analysis".to_string(),
                        public_api: PublicApi {
                            client: Vec::new(),
                            server: vec!["./server/public.ts".to_string()],
                            shared: Vec::new(),
                        },
                        depends_on: Vec::new(),
                    },
                ),
            ]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, RuleCode::Br001);
    }

    #[test]
    fn does_not_infer_compact_layer_for_nested_root_directory() {
        let imports = vec![ImportRecord {
            source_file: "domains/appearance-guidance/internal/helper.ts".to_string(),
            source_dir: "domains/appearance-guidance/internal".to_string(),
            line: 1,
            import_path: "react".to_string(),
        }];
        let context = BoundaryContext {
            apps_path: "apps".to_string(),
            domains_path: "domains".to_string(),
            packages_path: "packages".to_string(),
            domains: BTreeMap::from([(
                "appearance-guidance".to_string(),
                DomainManifest {
                    name: "appearance-guidance".to_string(),
                    public_api: PublicApi {
                        client: Vec::new(),
                        server: Vec::new(),
                        shared: vec!["./public.ts".to_string()],
                    },
                    depends_on: Vec::new(),
                },
            )]),
            path_aliases: Vec::new(),
        };

        let violations = check_boundaries_with_context(&imports, &context);
        assert!(violations.is_empty());
    }
}
