# 00-OVERNIGHT-RUNBOOK

## Mission

Advance **switchyard** without breaking deterministic guarantees or repo coherence.

## First pass

1. Run `cargo test --workspace --all-features` and `python3 scripts/validate_contract_fixtures.py` to establish the baseline.
2. Read `docs/02-TECHNICAL-ARCHITECTURE.md` and confirm the scheduler invariants before editing runtime code.
3. Run `python3 scripts/run_prompt_pack.py` to resolve the next prompt sequence from `codex/taskboard.yaml`.
4. Write a failing test first, then implement the smallest deterministic change.

## Execution loop

1. Execute the prompt sequence emitted by `scripts/run_prompt_pack.py`.
2. Move exactly one coherent slice forward.
3. Re-run the validation loop:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace --all-features`
   - `cargo check --workspace --lib --no-default-features`
   - `python3 scripts/validate_contract_fixtures.py`
4. Update `codex/taskboard.yaml` and `docs/05-ACCEPTANCE-TEST-MATRIX.md`.
5. Re-run `python3 scripts/run_prompt_pack.py` until the taskboard is done.

## Execution rules

- Inspect repo state before acting.
- Preserve any partially completed but coherent work.
- Write tests first for every functional change.
- End each session by updating `codex/taskboard.yaml` and the acceptance matrix.
- Do not widen scope without touching the risk register and milestone plan.

## Stop conditions

- A required invariant is unclear and would force speculative architecture.
- CI commands can no longer be made consistent with repo state.
- A new dependency would change the no_std or deterministic core story without spec updates.
