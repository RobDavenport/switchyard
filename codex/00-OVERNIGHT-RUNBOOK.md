# 00-OVERNIGHT-RUNBOOK

## Mission

Advance **switchyard** without breaking deterministic guarantees or repo coherence.

## First pass

1. Run `cargo test --workspace --all-features` and `python3 scripts/validate_contract_fixtures.py` to establish the baseline.
2. Read `docs/02-TECHNICAL-ARCHITECTURE.md` and confirm the scheduler invariants before editing runtime code.
3. Choose the highest-priority non-done item from `codex/taskboard.yaml`.
4. Write a failing test first, then implement the smallest deterministic change.

## Execution rules

- Inspect repo state before acting.
- Preserve any partially completed but coherent work.
- Write tests first for every functional change.
- End each session by updating `codex/taskboard.yaml` and the acceptance matrix.
- Do not widen scope without touching the risk register and milestone plan.

## Stop conditions

- A required invariant is unclear and would force speculative architecture
- CI commands can no longer be made consistent with repo state
- A new dependency would change the no_std or deterministic core story without spec updates
