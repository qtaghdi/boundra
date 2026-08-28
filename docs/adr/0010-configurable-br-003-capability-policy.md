# ADR 0010: Configurable BR-003 Capability Policy

- Status: Accepted
- Date: 2026-08-28

## Context

BR-003 protects `shared` as a pure contract layer, but its implementation
hardcoded framework- and infrastructure-specific dependencies. Extending the
rule for another UI framework, database client, or workspace layout required a
Boundra release.

The existing classifications must remain stable for projects that do not opt
into configuration.

## Decision

- Keep BR-003 and its diagnostic code.
- Classify dependencies with `checkBoundaries.capabilities`:
  - `external`: external import matcher to capability list
  - `packages`: direct child under `paths.packages` to capability list
  - `apps`: capabilities assigned to imports resolving into `paths.apps`
- Configure shared-layer denials with
  `checkBoundaries.policy.shared.denyCapabilities`.
- Preserve current behavior as defaults: React/Next map to `ui`, Prisma to
  `database`, Node modules and app paths to `runtime`, and the `ui`, `db`, and
  `infra` workspace packages keep their corresponding classifications.
- Overlay configured external/package entries on defaults. An empty list
  explicitly disables a default source. Explicit app and shared policy lists
  replace their defaults.
- Validate capability names, wildcard syntax, and package keys at config load.

## Consequences

Projects can model their technology stack without modifying the rule engine,
while existing projects retain the same behavior. The larger configuration
surface can weaken or over-constrain BR-003 if maintained incorrectly, so the
effective policy must remain reviewable in source control.

## Compatibility

All new fields are optional. BR-003 exit behavior and machine-readable
diagnostic shape remain unchanged; only its human-readable wording now refers
to a policy-denied capability.
