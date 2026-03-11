# Prompt instructions

- Inspect the current repo state before editing anything.
- Read the relevant spec docs and existing tests first.
- Continue from partial progress; do not redo completed work.
- Keep TDD discipline: create or update a failing test first.
- Preserve deterministic behavior and no_std constraints.
- End with concrete verification steps and any docs/taskboard updates required.

Focus: harden `contracts/` and `fixtures/`.

Actions:
- Review the current JSON Schema files and fixture validator script.
- Add missing required fields or invariants only with matching valid and invalid fixtures.
- Preserve backward-conscious contract evolution notes in docs.

Verification:
- `python3 scripts/validate_contract_fixtures.py`
- Update `docs/05-ACCEPTANCE-TEST-MATRIX.md` if new contract coverage is added
