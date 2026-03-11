# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: final documentation coherence after code changes.

Actions:
- Update docs to match the real runtime behavior.
- Mark incomplete features honestly.
- Keep the taskboard, acceptance matrix, README, and MASTER_SPEC aligned.

Verification:
- Manual audit of README, MASTER_SPEC, docs, and taskboard
- Re-run `make ci`
