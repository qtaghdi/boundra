# ADR 0011: Manifest-Backed Nested Domain Discovery

- Status: Accepted
- Date: 2026-09-14

## Context

Boundra previously treated each direct child of `paths.domains` as a domain.
That assumption prevents teams from grouping domains under organizational
directories such as `domains/commerce/order` and couples domain identity to a
single flat filesystem layout.

Boundary checks, generation, dependency updates, and coverage reporting also
reconstructed a domain root as `paths.domains/<name>`. Once domains may be
nested, that reconstruction can target the wrong directory and can classify a
group directory as a domain.

## Decision

- Boundra discovers domains by recursively locating the configured
  `domain.manifestFile` below `paths.domains`.
- The manifest `name` remains the stable domain identity. Moving a domain
  between grouping directories does not rename it.
- A manifest's parent directory is recorded as that domain's root and is used
  by boundary checks, coverage reporting, generation, and dependency updates.
- The manifest name must continue to match the leaf domain directory name.
  Parent directories are organizational only.
- Duplicate manifest names are a project configuration error. The diagnostic
  lists both roots in deterministic path order.
- Directories without a domain manifest are not domains. This prevents grouping
  directories from acquiring accidental public APIs or boundary identities.
- `create-domain <name> --path <parent-path>` creates the domain below a
  relative, kebab-case parent path under `paths.domains`.
- Existing manifest-backed flat domains continue to work unchanged.

## Consequences

Positive:

- related domains can be grouped without changing their logical names
- all commands share one discovered source of truth for physical domain roots
- duplicate identities and missing manifests fail early instead of producing
  partial or misleading analysis
- custom manifest filenames work at every nesting depth

Negative:

- legacy directories that relied on implicit manifests must add the configured
  manifest file before they are treated as domains
- tools integrating directly with the Rust project model must account for the
  new domain-root map

## Migration

1. Add the configured manifest file to every directory that should be a domain.
2. Move domain directories below optional kebab-case grouping directories.
3. Keep each manifest `name` equal to the leaf domain directory name.
4. Run `boundra check-boundaries` and `boundra graph-domains`.
