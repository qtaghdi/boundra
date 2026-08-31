use std::path::PathBuf;

use boundra_core::load_config;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/config-conformance")
}

#[test]
fn shared_config_fixture_matches_native_loader() {
    let config = load_config(&fixture_root()).expect("shared config fixture should load");

    assert_eq!(config.project.workspace_root, "workspace");

    assert_eq!(config.paths.apps, "products");
    assert_eq!(config.paths.domains, "bounded-contexts");
    assert_eq!(config.paths.packages, "libraries");
    assert_eq!(config.paths.crates, "native");

    assert_eq!(config.domain.manifest_file, "boundra-domain.json");
    assert_eq!(config.domain.public_api.client, vec!["./browser/public.ts"]);
    assert_eq!(config.domain.public_api.server, vec!["./backend/public.ts"]);
    assert_eq!(config.domain.public_api.shared, vec!["./contract/public.ts"]);

    assert_eq!(
        config.check_boundaries.include_extensions,
        vec!["ts", "tsx", "svelte"]
    );
    assert_eq!(
        config.check_boundaries.ignore,
        vec!["**/generated/**", "**/.cache/**"]
    );

    assert_eq!(
        config
            .check_boundaries
            .capabilities
            .external
            .get("react")
            .cloned(),
        Some(vec!["ui".to_string(), "browser".to_string()])
    );
    assert_eq!(
        config
            .check_boundaries
            .capabilities
            .external
            .get("node:*")
            .cloned(),
        Some(vec!["runtime".to_string(), "node".to_string()])
    );
    assert_eq!(
        config
            .check_boundaries
            .capabilities
            .packages
            .get("persistence")
            .cloned(),
        Some(vec!["database".to_string()])
    );
    assert_eq!(
        config
            .check_boundaries
            .capabilities
            .packages
            .get("design-system")
            .cloned(),
        Some(vec!["ui".to_string()])
    );
    assert_eq!(
        config.check_boundaries.capabilities.apps,
        vec!["runtime", "application"]
    );
    assert_eq!(
        config.check_boundaries.policy.shared.deny_capabilities,
        vec!["ui", "database", "runtime", "application"]
    );
}
