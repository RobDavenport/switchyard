# AGENTS.md

## Repo working rules

- Read `MASTER_SPEC.md`, then the relevant file in `docs/`, before changing code.
- Treat the taskboard as the source of truth for sequencing.
- Preserve existing interfaces unless a failing test or spec update justifies a change.
- Prefer additive, reversible changes over broad rewrites.
- Keep commits logically scoped by work package.

## Architecture invariants

- The repo must remain a Rust workspace.
- The core runtime path must stay `no_std` compatible.
- Boundary contracts in `contracts/` must remain versioned and test-backed.
- Deterministic behavior is a release blocker.
- Avoid hidden globals, hidden I/O, and framework-shaped abstractions in core crates.

## TDD expectations

- Default loop: **Red -> Green -> Refactor**.
- Start with the narrowest failing test that proves the requirement.
- Do not add production code without a test or contract that justifies it.
- Prefer table-driven tests for edge cases and invariant tests for runtime semantics.
- When a bug is fixed, add a regression test before or alongside the fix.

## Forbidden shortcuts

- No `todo!()`, `unimplemented!()`, or fake happy-path shims in shipped code.
- No hidden heap allocation in the hot path unless a feature gate and design note justify it.
- No weakening deterministic ordering rules to “whatever the container gives us.”
- No claiming a milestone is finished unless tests and CI back it up.

## File ownership / subsystem boundaries

- `crates/`: production Rust code and tests
- `contracts/`: externalized contracts and schemas
- `fixtures/`: valid and invalid examples
- `docs/`: product and engineering intent
- `codex/`: agent runbook, prompts, and execution tracking
- `scripts/`: deterministic helper scripts only

## Validation checklist before claiming completion

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo check --workspace --lib --no-default-features`
- `python3 scripts/validate_contract_fixtures.py`
- Update acceptance matrix and taskboard status where relevant

## How to resume from partial repo state

- Inspect `codex/taskboard.yaml` for the latest seeded status.
- Inspect existing tests before touching implementation files.
- Continue from the highest-priority item that is not `done`.
- Preserve partial progress; do not delete incomplete but coherent work unless a spec mismatch requires replacement.
