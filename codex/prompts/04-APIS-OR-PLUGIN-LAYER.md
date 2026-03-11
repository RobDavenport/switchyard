# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: public API clarity, examples, and future layering.

Actions:
- Improve public API names and docs without widening the framework surface.
- Extend the example only if it stays aligned with the current walking skeleton.
- Keep host integration explicit; do not introduce reflection or hidden registries.

Verification:
- `cargo test -p switchyard-core --all-features`
- Review README + example alignment
