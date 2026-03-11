# MASTER_SPEC.md

## Product intent

- **Product:** switchyard
- **Repo slug:** switchyard
- **Description:** Deterministic, no_std structured-concurrency behavior runtime for game logic and orchestration.
- **Users:** Engine authors, gameplay programmers, simulation developers, custom runtime builders, and wasm-hosted game teams.
- **Problem:** Long-lived gameplay behavior is painful to express as scattered tick code, callback chains, and ad hoc state machines. Teams need a game-first orchestration layer without adopting a full framework or embedding a heavyweight scripting language.
- **Core value proposition:** A small, deterministic runtime for durational behavior, structured concurrency, inspection, and save/load-friendly execution state.

## Architecture summary

- Rust workspace root with a focused core crate under `crates/`
- `no_std`-compatible core path
- Contract fixtures and schemas under `contracts/` + `fixtures/`
- Agent execution pack under `codex/`
- CI-ready workflow under `.github/workflows/`

## Constraints

- solo-dev friendly, no_std compatible, deterministic, performance-sensitive, narrow host boundary, minimal mandatory dependencies, permissive licensing
- Strong typing and deterministic behavior take precedence over convenience APIs.
- The repository must stay small enough for solo maintenance.
- Docs and tests must be specific to the product, not generic templates.

## Milestone shape

1. Workspace + tooling baseline
2. Core domain walking skeleton
3. Boundary contracts + deterministic fixture validation
4. Examples + acceptance tests
5. CI + release hygiene

## Acceptance criteria

- The workspace is coherent and ready for agent-driven continuation.
- Core crate contains real code and real tests.
- Every must-have requirement is mapped in `docs/05-ACCEPTANCE-TEST-MATRIX.md`.
- Commands in the README and CI are aligned.
- The current state is honestly labeled as scaffolded, partially implemented, or complete.

## Implementation priorities

1. Preserve deterministic semantics
2. Keep the core API small and explicit
3. Lock boundaries before widening functionality
4. Expand from proven tests, not speculative abstractions

## Risks

- Scope creep from helper library into language or framework
- Weak contract validation leading to unstable external surfaces
- Overdesign before the walking skeleton proves the semantics

## Open assumptions

- Rust stable remains the default toolchain for contributors.
- Python 3 is available in CI and local development environments.
- The first release optimizes for clarity and determinism over feature breadth.
