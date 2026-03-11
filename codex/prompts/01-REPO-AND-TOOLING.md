# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: keep repo hygiene and commands aligned.

Actions:
- Inspect root files, CI workflow, and Makefile for drift.
- Add or refine developer automation only if it removes ambiguity.
- Do not introduce third-party tooling unless it materially improves deterministic validation and is documented.

Verification:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
