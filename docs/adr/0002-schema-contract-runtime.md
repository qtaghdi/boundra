# ADR 0002: Schema-Backed Contract Runtime

- Status: Accepted
- Date: 2026-07-01

## Context

The first dogfood flow proved that generated files, manifests, aliases, and
boundary checks connect correctly. It also exposed that
`Record<string, never>` contracts and placeholder client hooks only validate
wiring; they do not provide a useful application framework surface.

Changing generated contract and adapter shapes affects the CLI's generated
interface, so the decision is recorded before implementation.

## Decision

- Generated shared contracts define runtime input and result schemas with Zod.
- `boundra` accepts schemas structurally through a small
  `BoundraSchema<T>` interface and does not import Zod itself.
- Contract types are inferred from schemas; generated TypeScript does not keep a
  second handwritten type definition.
- Route implementations validate input and result values at runtime.
- Query and mutation adapters use a framework-neutral `BoundraClient` and
  `BoundraTransport`; React, HTTP, and server framework bindings remain outside
  the core runtime.
- Code generation maintains `shared/public.ts` exports in addition to retaining
  direct manifest entries for backward compatibility.
- Generated query and mutation client files are ordinary async functions, not
  fake React hooks.

## Consequences

Positive:

- contracts become executable runtime boundaries rather than type-only markers
- input and result types cannot drift away from their schemas
- apps can use one transport abstraction without Boundra owning HTTP or React
- public contract imports become stable through a generated barrel

Negative:

- projects using generated contracts need Zod
- existing placeholder generated files require migration
- generated client filenames and exports change before the external v1 release

## Follow-up

- dogfood route/query/mutation execution through the runtime
- specify framework adapters only after a concrete Next.js or other host flow
- keep schema-provider abstraction narrow; do not build a Boundra schema library
