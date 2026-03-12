# 07-REPO-BLUEPRINT

## Full repo tree

```text
switchyard/
„¥„Ÿ„Ÿ .github/
„    „¤„Ÿ„Ÿ workflows/
„        „¥„Ÿ„Ÿ ci.yml
„        „¤„Ÿ„Ÿ pages.yml
„¥„Ÿ„Ÿ codex/
„    „¥„Ÿ„Ÿ 00-OVERNIGHT-RUNBOOK.md
„    „¥„Ÿ„Ÿ ENVIRONMENT-NOTES.md
„    „¥„Ÿ„Ÿ prompts/
„    „    „¥„Ÿ„Ÿ 00-LAUNCH-THIS-REPO.md
„    „    „¥„Ÿ„Ÿ 01-REPO-AND-TOOLING.md
„    „    „¥„Ÿ„Ÿ 02-CONTRACTS-AND-SCHEMAS.md
„    „    „¥„Ÿ„Ÿ 03-CORE-DOMAIN.md
„    „    „¥„Ÿ„Ÿ 04-APIS-OR-PLUGIN-LAYER.md
„    „    „¥„Ÿ„Ÿ 05-TESTS-AND-VALIDATION.md
„    „    „¥„Ÿ„Ÿ 06-CI-LINT-AND-RELEASE.md
„    „    „¤„Ÿ„Ÿ 07-DOCS-FINAL-AUDIT.md
„    „¤„Ÿ„Ÿ taskboard.yaml
„¥„Ÿ„Ÿ contracts/
„    „¥„Ÿ„Ÿ behavior-program.schema.json
„    „¤„Ÿ„Ÿ runtime-snapshot.schema.json
„¥„Ÿ„Ÿ crates/
„    „¥„Ÿ„Ÿ switchyard-core/
„    „    „¥„Ÿ„Ÿ examples/
„    „    „    „¤„Ÿ„Ÿ cutscene.rs
„    „    „¥„Ÿ„Ÿ src/
„    „    „    „¥„Ÿ„Ÿ ids.rs
„    „    „    „¥„Ÿ„Ÿ lib.rs
„    „    „    „¥„Ÿ„Ÿ program.rs
„    „    „    „¥„Ÿ„Ÿ runtime.rs
„    „    „    „¥„Ÿ„Ÿ snapshot.rs
„    „    „    „¤„Ÿ„Ÿ trace.rs
„    „    „¤„Ÿ„Ÿ tests/
„    „        „¥„Ÿ„Ÿ owned_program.rs
„    „        „¥„Ÿ„Ÿ program_builder.rs
„    „        „¥„Ÿ„Ÿ smoke.rs
„    „        „¤„Ÿ„Ÿ trace.rs
„    „¤„Ÿ„Ÿ switchyard-debug/
„        „¥„Ÿ„Ÿ src/
„        „    „¤„Ÿ„Ÿ lib.rs
„        „¤„Ÿ„Ÿ tests/
„            „¤„Ÿ„Ÿ trace_log.rs
„¥„Ÿ„Ÿ demo-wasm/
„    „¥„Ÿ„Ÿ src/
„    „    „¤„Ÿ„Ÿ lib.rs
„    „¥„Ÿ„Ÿ tests/
„    „    „¤„Ÿ„Ÿ showcase.rs
„    „¥„Ÿ„Ÿ www/
„    „    „¥„Ÿ„Ÿ index.html
„    „    „¥„Ÿ„Ÿ main.js
„    „    „¤„Ÿ„Ÿ styles.css
„    „¥„Ÿ„Ÿ Cargo.toml
„    „¤„Ÿ„Ÿ README.md
„¥„Ÿ„Ÿ docs/
„    „¥„Ÿ„Ÿ 01-PRD.md
„    „¥„Ÿ„Ÿ 02-TECHNICAL-ARCHITECTURE.md
„    „¥„Ÿ„Ÿ 03-WBS-AND-MILESTONES.md
„    „¥„Ÿ„Ÿ 04-TDD-QUALITY-GATES.md
„    „¥„Ÿ„Ÿ 05-ACCEPTANCE-TEST-MATRIX.md
„    „¥„Ÿ„Ÿ 06-RISK-REGISTER.md
„    „¤„Ÿ„Ÿ 07-REPO-BLUEPRINT.md
„¥„Ÿ„Ÿ fixtures/
„    „¤„Ÿ„Ÿ contracts/
„        „¥„Ÿ„Ÿ program.invalid.json
„        „¥„Ÿ„Ÿ program.valid.json
„        „¥„Ÿ„Ÿ snapshot.invalid.json
„        „¤„Ÿ„Ÿ snapshot.valid.json
„¥„Ÿ„Ÿ scripts/
„    „¥„Ÿ„Ÿ run_prompt_pack.py
„    „¥„Ÿ„Ÿ test_run_prompt_pack.py
„    „¤„Ÿ„Ÿ validate_contract_fixtures.py
„¥„Ÿ„Ÿ AGENTS.md
„¥„Ÿ„Ÿ Cargo.toml
„¥„Ÿ„Ÿ Makefile
„¥„Ÿ„Ÿ MASTER_SPEC.md
„¤„Ÿ„Ÿ README.md
```

## Top-level directory purposes

- `crates/`: production Rust workspace members
- `demo-wasm/`: browser showcase crate and static site assets
- `contracts/`: versioned external schemas and boundary documentation
- `fixtures/`: valid and invalid contract examples
- `docs/`: product, architecture, quality, and delivery guidance
- `codex/`: agent runbook, prompts, prompt-loop entry point, and task tracking
- `scripts/`: deterministic helper scripts for contract validation and prompt-pack execution
- `.github/workflows/`: CI and Pages deployment definitions

## Naming conventions

- Workspace members use the `switchyard-*` prefix unless the crate is a purpose-built showcase app.
- Contracts use kebab-case file names ending in `.schema.json`.
- Prompt files are numbered to keep resumption deterministic.
- Helper scripts stay deterministic and standard-library-only.

## Future extension points

- Add new crates only when they own a stable boundary.
- Keep examples, fixtures, and the browser showcase aligned with real acceptance cases.
- Prefer sibling crates for optional tooling instead of bloating the core runtime.
