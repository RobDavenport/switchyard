# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: strengthen correctness and regression coverage.

Actions:
- Add scenario tests for edge cases that are now underspecified.
- Add regression tests for any bug fixed in this run.
- Prefer deterministic table-driven cases over brittle one-off assertions.

Verification:
- `cargo test --workspace --all-features`
- `python3 scripts/validate_contract_fixtures.py`
