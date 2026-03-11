    # switchyard

    Deterministic, no_std structured-concurrency behavior runtime for game logic and orchestration.

    ## Project purpose

    Long-lived gameplay behavior is painful to express as scattered tick code, callback chains, and ad hoc state machines. Teams need a game-first orchestration layer without adopting a full framework or embedding a heavyweight scripting language.

    switchyard exists to provide a small, deterministic runtime for durational behavior, structured concurrency, inspection, and save/load-friendly execution state while staying reusable across engines, runtimes, and architectural styles.

    ## Users

    Engine authors, gameplay programmers, simulation developers, custom runtime builders, and wasm-hosted game teams.

    ## Delivery mode

    **scaffold + walking skeleton**

    This repository is intentionally narrow. It ships a compileable workspace, a working skeleton in code, boundary contracts, starter tests, CI wiring, and an agent execution pack. It does **not** claim the full product is complete.

    ## Repo layout

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

    ## Prerequisites

    - Rust stable toolchain with `clippy` and `rustfmt`
    - Python 3.11+ for contract-fixture validation scripts
    - Standard POSIX shell environment for local automation

    ## Setup commands

    ```bash
    git clone <your-fork-url> switchyard
    cd switchyard
    make bootstrap
    make test
    ```

    ## Common commands

    ```bash
    make fmt
    make lint
    make test
    make test-no-default
    make docs
    ```

    ## Development workflow

    1. Pick the next open item from `codex/taskboard.yaml`.
    2. Write or extend a failing test first.
    3. Implement the smallest change that turns the test green.
    4. Refactor only after the behavior is locked by tests.
    5. Re-run `make ci` before claiming completion.
    6. Update the docs pack and taskboard when scope or status changes.

    ## How an agent should start

    1. Read `MASTER_SPEC.md`.
    2. Read `AGENTS.md`.
    3. Open `codex/00-OVERNIGHT-RUNBOOK.md`.
    4. Execute `codex/prompts/00-LAUNCH-THIS-REPO.md`.
    5. Continue through the numbered prompt pack without redoing finished work.

    ## Preferred stack

    Rust stable workspace, additive `std` feature wiring, no required third-party runtime dependencies.

    ## What is scaffolded vs implemented

    Implemented now: workspace scaffold, `switchyard-core`, a functioning scheduler skeleton, snapshot model, contract schemas, fixture validation, CI, example, a fixed-capacity program builder API, explicit runtime trace hooks, and behavior tests for waits, join, race, predicates, restore-from-snapshot, authoring, and tracing.

    Partially implemented: richer authoring surfaces beyond the fixed-capacity builder, higher-level debug tooling on top of trace events, more expressive routine formats, and optional serde-backed export/import.

    ## Next milestones

    1. Harden the scheduler edge cases and failure semantics.
    2. Expand the program surface beyond the fixed-capacity builder with richer authoring helpers or a tiny assembler.
    3. Add higher-level debug/trace tooling on top of the runtime event hooks and more representative acceptance examples.
    4. Introduce optional serde integration only after the snapshot format is stable.
