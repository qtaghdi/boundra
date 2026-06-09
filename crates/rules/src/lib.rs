use boundra_core::{Layer, RuleCode, Violation};
use boundra_parser::ImportRecord;

pub fn check_boundaries(imports: &[ImportRecord]) -> Vec<Violation> {
    let mut violations = Vec::new();

    for record in imports {
        let source = parse_domain_path(&record.source_file);
        let (source_domain, source_layer) = source
            .clone()
            .map_or((String::new(), Layer::Unknown), |(domain, layer)| {
                (domain, layer)
            });

        let resolved_target = resolve_import_path(&record.source_dir, &record.import_path);
        let Some(target) = resolved_target else {
            continue;
        };
        let target_parsed = parse_domain_path(&target);
        let (target_domain, target_layer) = target_parsed
            .clone()
            .map_or((String::new(), Layer::Unknown), |(domain, layer)| {
                (domain, layer)
            });

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

        if is_cross_domain_internal_import(&source_domain, &target_domain, &target) {
            violations.push(Violation {
                rule: RuleCode::Br004,
                file: record.source_file.clone(),
                line: record.line,
                import_path: record.import_path.clone(),
                message: "domains cannot import another domain's internal path".to_string(),
                suggestion: "import from the target domain's public API instead".to_string(),
            });
        }
    }

    violations
}

pub fn resolve_import_path(source_dir: &str, import_path: &str) -> Option<String> {
    if import_path.starts_with("domains/") {
        return Some(normalize_path(import_path));
    }
    if !import_path.starts_with('.') {
        return None;
    }

    let joined = if source_dir.is_empty() {
        import_path.to_string()
    } else {
        format!("{source_dir}/{import_path}")
    };

    Some(normalize_path(&joined))
}

fn parse_domain_path(path: &str) -> Option<(String, Layer)> {
    let normalized = normalize_path(path);
    let mut parts = normalized.split('/');

    let first = parts.next()?;
    if first != "domains" {
        return None;
    }

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

fn is_cross_domain_internal_import(
    source_domain: &str,
    target_domain: &str,
    target_path: &str,
) -> bool {
    if source_domain.is_empty() || target_domain.is_empty() || source_domain == target_domain {
        return false;
    }

    !is_public_api_path(target_domain, target_path)
}

fn is_public_api_path(domain: &str, target_path: &str) -> bool {
    let normalized_path = normalize_path(target_path);
    let normalized = strip_ts_like_extension(&normalized_path);
    normalized == format!("domains/{domain}/shared/public")
}

fn strip_ts_like_extension(path: &str) -> &str {
    for extension in [".ts", ".tsx", ".js", ".jsx"] {
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
}
