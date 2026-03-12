# switchyard

Deterministic, `no_std` structured-concurrency behavior runtime for game logic and orchestration.

## Project purpose

Long-lived gameplay behavior is painful to express as scattered tick code, callback chains, and ad hoc state machines. `switchyard` provides a small, deterministic runtime for durational behavior, structured concurrency, inspection, tracing, and save/load-friendly execution state without imposing a framework.

## Delivery mode

This repo now contains a working walking skeleton with a browser showcase. The core runtime, authoring helpers, contracts, trace hooks, debug tooling, CI checks, GitHub Pages showcase, and agent workflow loop are implemented and test-backed.

## Workspace layout

- `crates/switchyard-core`: deterministic runtime, snapshots, trace events, and authoring helpers
- `crates/switchyard-debug`: trace log sink and rendering helpers built on top of `switchyard-core`
- `demo-wasm`: browser showcase inspired by gthe death of tickh, including snapshots and trace playback
- `contracts/`: versioned JSON Schema boundary contracts
- `fixtures/`: valid and invalid contract examples
- `docs/`: product, architecture, quality, and release guidance
- `codex/`: runbook, prompt pack, and taskboard
- `scripts/`: deterministic helper scripts, including the prompt-pack loop runner

## Prerequisites

- Rust stable with `clippy` and `rustfmt`
- Python 3.11+
- `wasm-pack` for the browser showcase
- Standard POSIX shell environment for the `Makefile`

## Common commands

```bash
make fmt
make lint
make test
make test-no-default
make docs
make runpack
make showcase-wasm
make ci
```

`make runpack` resolves the current prompt-pack loop from `codex/taskboard.yaml`.

## Browser showcase

The repo includes a browser demo in `demo-wasm/` that stages a branching encounter with:

- explicit ticks
- signal buttons
- a predicate toggle
- live task inspection
- trace log playback
- snapshot save/restore

Build the package and serve the static site locally:

```bash
wasm-pack build demo-wasm --target web --release --out-dir www/pkg
cd demo-wasm/www
python -m http.server 8080
```

The GitHub Pages deployment workflow lives in `.github/workflows/pages.yml`.

## Agent workflow

1. Read `MASTER_SPEC.md` and `AGENTS.md`.
2. Read `codex/00-OVERNIGHT-RUNBOOK.md`.
3. Execute `codex/prompts/00-LAUNCH-THIS-REPO.md`.
4. Run `python3 scripts/run_prompt_pack.py`.
5. Move one coherent slice forward with Red -> Green -> Refactor.
6. Update the taskboard and acceptance matrix.
7. Re-run `python3 scripts/run_prompt_pack.py` until the taskboard is done.

## Implemented now

- Fixed-capacity deterministic scheduler with waits, signals, predicates, spawn, join, race, fail, and cancellation
- Snapshot export/import and plain-data task inspection
- Fixed-capacity `ProgramBuilder` for `no_std` authoring
- Alloc-backed `OwnedProgram` for dynamic authoring when `alloc` is available
- Optional `serde` feature for snapshot JSON round-trip and `OwnedProgram` export/import
- Explicit `TraceEvent` / `TraceSink` runtime hooks
- `switchyard-debug::TraceLog` for reusable event capture and rendering
- Browser showcase for orchestrated encounter stepping and snapshot/trace inspection
- Contract schemas, valid/invalid fixtures, CI, GitHub Pages deployment, and prompt-pack loop automation

## Remaining expansion areas

- Richer authoring formats above op-by-op builders
- More representative acceptance examples and sample integrations

## Verification

The repo is expected to stay green on:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --workspace --lib --no-default-features
python3 -m unittest scripts/test_run_prompt_pack.py
python3 scripts/validate_contract_fixtures.py
```
