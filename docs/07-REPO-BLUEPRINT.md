        # 07-REPO-BLUEPRINT

        ## Full repo tree

        ```text
        switchyard-ready-monorepo
├── .editorconfig
├── .github/
│   └── workflows/
│       └── ci.yml
├── .gitignore
├── AGENTS.md
├── Cargo.toml
├── LICENSE
├── MASTER_SPEC.md
├── Makefile
├── README.md
├── clippy.toml
├── codex/
│   ├── 00-OVERNIGHT-RUNBOOK.md
│   ├── ENVIRONMENT-NOTES.md
│   ├── prompts/
│   │   ├── 00-LAUNCH-THIS-REPO.md
│   │   ├── 01-REPO-AND-TOOLING.md
│   │   ├── 02-CONTRACTS-AND-SCHEMAS.md
│   │   ├── 03-CORE-DOMAIN.md
│   │   ├── 04-APIS-OR-PLUGIN-LAYER.md
│   │   ├── 05-TESTS-AND-VALIDATION.md
│   │   ├── 06-CI-LINT-AND-RELEASE.md
│   │   └── 07-DOCS-FINAL-AUDIT.md
│   └── taskboard.yaml
├── contracts/
│   ├── behavior-program.schema.json
│   └── runtime-snapshot.schema.json
├── crates/
│   └── switchyard-core/
│       ├── Cargo.toml
│       ├── examples/
│       │   └── cutscene.rs
│       ├── src/
│       │   ├── ids.rs
│       │   ├── lib.rs
│       │   ├── program.rs
│       │   ├── runtime.rs
│       │   └── snapshot.rs
│       └── tests/
│           └── smoke.rs
├── docs/
│   ├── 01-PRD.md
│   ├── 02-TECHNICAL-ARCHITECTURE.md
│   ├── 03-WBS-AND-MILESTONES.md
│   ├── 04-TDD-QUALITY-GATES.md
│   ├── 05-ACCEPTANCE-TEST-MATRIX.md
│   ├── 06-RISK-REGISTER.md
│   └── 07-REPO-BLUEPRINT.md
├── fixtures/
│   └── contracts/
│       ├── program.invalid.json
│       ├── program.valid.json
│       ├── snapshot.invalid.json
│       └── snapshot.valid.json
├── rust-toolchain.toml
├── rustfmt.toml
└── scripts/
    └── validate_contract_fixtures.py
        ```

        ## Top-level directory purposes

        - `crates/`: production Rust workspace members
        - `contracts/`: versioned external schemas and boundary documentation
        - `fixtures/`: valid and invalid contract examples
        - `docs/`: product, architecture, quality, and delivery guidance
        - `codex/`: agent runbook, prompts, and task tracking
        - `scripts/`: deterministic developer and CI helper scripts
        - `.github/workflows/`: CI definitions

        ## Naming conventions

        - Workspace members use the product prefix (`switchyard-*`) to keep ownership obvious.
        - Contracts use kebab-case file names ending in `.schema.json`.
        - Prompts are numbered so agents can resume from partial progress without re-planning the whole repo.

        ## Future extension points

        - Add new crates only when they own a stable boundary.
        - Keep examples and fixtures aligned with real acceptance cases.
        - Prefer sibling crates for optional tooling instead of bloating the core crate.
