# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: CI quality gates and release readiness.

Actions:
- Confirm CI commands match the README and Makefile.
- Keep no-default-features validation intact.
- Add release metadata only if it reflects the actual scaffold maturity.

Verification:
- Review `.github/workflows/ci.yml`
- `make ci`
