# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: improve the scheduler and core runtime semantics.

Actions:
- Inspect existing tests under `crates/switchyard-core/tests/`.
- Add one failing test for the next deterministic runtime behavior.
- Implement the minimal runtime change required to pass that test.
- Refactor only after the new and old tests are green.

Verification:
- `cargo test -p switchyard-core --all-features`
- `cargo check -p switchyard-core --lib --no-default-features`
