# 04-TDD-QUALITY-GATES

## Required test layers

- Unit tests for core invariants and edge conditions
- Scenario or integration tests for the walking skeleton
- Contract-fixture validation under `scripts/validate_contract_fixtures.py`
- Regression tests for every bug fixed after the scaffold state
- Documentation examples that match public APIs

## Default development loop

1. Start with a failing test or failing contract-fixture assertion.
2. Implement the smallest production change that makes the new test pass.
3. Refactor only after all tests are green.
4. Re-run formatting, linting, tests, and contract validation before moving the taskboard status to `done`.

## Coverage expectations

- Core deterministic rules must be covered by direct tests.
- Every acceptance-matrix row must point to at least one test, fixture, or explicit TODO status.
- New public APIs require at least one behavior test and one edge-case test.

## Boundary validation rules

- External schemas in `contracts/` must have valid and invalid fixtures.
- Serialization formats and config surfaces must remain backward-conscious; changes require fixture updates.
- Deterministic ordering rules must be asserted in tests, not merely described in docs.

## CI quality gates

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo check --workspace --lib --no-default-features`
- `python3 scripts/validate_contract_fixtures.py`

## Release criteria

- All required CI gates pass
- Acceptance matrix rows for the target milestone are green
- README, MASTER_SPEC, and taskboard agree on current scope
- No known deterministic correctness issue remains open for the release target
