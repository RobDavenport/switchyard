# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Goal: confirm the repository is coherent, reproducible, and ready for iterative agent work.

Steps:
1. Read `MASTER_SPEC.md`, `AGENTS.md`, and `docs/02-TECHNICAL-ARCHITECTURE.md`.
2. Run the baseline verification commands.
3. Summarize what is already implemented, what is scaffold-only, and the next highest-value gap.
4. Pick exactly one work package slice and move it forward using TDD.
5. Update the taskboard and acceptance matrix if any status changes.

Verification:
- `cargo test --workspace --all-features`
- `cargo check --workspace --lib --no-default-features`
- `python3 scripts/validate_contract_fixtures.py`
