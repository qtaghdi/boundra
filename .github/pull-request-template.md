## Summary

- 

## Type

- [ ] `feat` - user-facing feature
- [ ] `fix` - bug fix
- [ ] `docs` - documentation
- [ ] `config` - tooling or project configuration
- [ ] `refactor` - internal structure change
- [ ] `test` - test coverage
- [ ] `chore` - maintenance

## Boundary Review

- [ ] No `client -> server` imports (BR-001)
- [ ] No `server -> client` imports (BR-002)
- [ ] `shared` stays pure (BR-003)
- [ ] Cross-domain imports use manifest-declared public APIs only (BR-004)
- [ ] Not applicable

## Documentation

- [ ] Source-of-truth docs are updated when behavior changes
- [ ] ADR added for architecture, boundary, CLI, or manifest breaking changes
- [ ] Not applicable

## Validation

- [ ] `pnpm verify-dogfood`
- [ ] `pnpm verify-clean-room`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo run -p boundra-cli -- check-boundaries --root .`
- [ ] Other:
- [ ] Not run:

## Notes

- 
