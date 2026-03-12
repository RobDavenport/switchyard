# switchyard

Deterministic, `no_std` structured-concurrency behavior runtime for game logic and orchestration.

## Project purpose

Long-lived gameplay behavior is painful to express as scattered tick code, callback chains, and ad hoc state machines. `switchyard` provides a small, deterministic runtime for durational behavior, structured concurrency, inspection, tracing, and save/load-friendly execution state without imposing a framework.

## Delivery mode

This repo now contains a working runtime, authoring surface, debug tooling, prompt-pack loop, browser showcase, representative gameplay samples, and a host-side inspection CLI. The core runtime, contract fixtures, trace hooks, explicit sync authoring, explicit race authoring, per-task Mind change scheduling, conditional branching, bounded repeat authoring, repeat-until predicate orchestration, signal-or-timeout waits, child timeout combinators, timeout-race child combinators, join-any child barriers, external host calls with operands, runtime-editable script compilation, visual script editor surface, CI checks, GitHub Pages demo, shootemup sample, multi-mind showcase, and catalog/snapshot CLI are implemented and test-backed.

## Workspace layout

- `crates/switchyard-core`: deterministic runtime, snapshots, trace events, and authoring helpers
- `crates/switchyard-cli`: std-only catalog/snapshot inspection commands for pipelines and editor tooling
- `crates/switchyard-debug`: trace log sink and rendering helpers built on top of `switchyard-core`
- `demo-wasm`: browser showcase inspired by "the death of tick", including encounter, shootemup, and multi-mind presets, snapshots, trace playback, and a visual script editor
- `contracts/`: versioned JSON Schema boundary contracts
- `fixtures/`: valid and invalid contract examples, including asset-bundle manifests
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

The repo includes a browser demo in `demo-wasm/` with three built-in presets:

- `Death of Tick`: branching encounter orchestration in the style of the article
- `Shootemup Boss`: boss-phase timing, projectile host calls, and a core-break versus enrage race
- `Mind the Gap`: director/gameplay mind handoff with parked tasks and explicit active-mind switching

Across all three presets, the demo exposes:

- explicit ticks
- signal buttons
- a predicate toggle
- visual program/op editing over the enum-backed script model
- live task inspection
- trace log playback
- snapshot save/restore

The browser editor already emits CLI-compatible catalog JSON through the same contract-shaped
document behind `script_json()`, and the CLI handoff panel can export both the current catalog and
a compatible runtime snapshot for paired validation with `switchyard-cli snapshot-check`.

Build the package and serve the static site locally:

```bash
wasm-pack build demo-wasm --target web --release --out-dir www/pkg
cd demo-wasm/www
python -m http.server 8080
```

The GitHub Pages deployment workflow lives in `.github/workflows/pages.yml`.

## Native shootemup example

The repo also includes a small native example in `crates/switchyard-core/examples/shootemup.rs` that drives the same enum-backed runtime from a console host. It demonstrates:

- host-call projectile bursts
- `sync_children` for concurrent windup and barrage setup
- `repeat_count` for repeated firing cadences
- `race_children_until_tick` for core-break versus enrage resolution

Run it locally with:

```bash
cargo run -p switchyard-core --example shootemup
```

## Native director handoff example

The repo also includes a small native example in `crates/switchyard-core/examples/director_handoff.rs` that shows the host boundary for cutscene-to-gameplay control transfer without the browser layer. It demonstrates:

- explicit `change_mind` handoff from director logic to gameplay logic
- parked tasks that do not advance until the host activates the target mind
- snapshot save/restore while the gameplay mind is parked
- deterministic replay of the same resumed handoff after restore

Run it locally with:

```bash
cargo run -p switchyard-core --example director_handoff
```

## Asset CLI

The repo also includes a host-side inspection tool in `crates/switchyard-cli` for catalog, snapshot, and asset bundle manifest assets. It exposes:

- `catalog-check <path>` to parse and compile a contract-shaped program catalog through the real `switchyard-core` authoring path
- `catalog-summary <path>` to emit deterministic JSON with program ids, references, op histogram, and used signals/predicates/minds/host calls
- `snapshot-summary <path>` to emit deterministic JSON with clock, task count, pending signals, wait-kind histogram, and outcome histogram
- `snapshot-check <catalog> <snapshot>` to validate that a runtime snapshot only references programs and tasks that are compatible with the supplied catalog
- `asset-bundle-summary <manifest>` to emit deterministic JSON for a validated asset bundle manifest, including catalog program ids plus per-snapshot clocks, task counts, pending signal counts, and used program/mind ids
- `catalog-normalize <input> [output]` to rewrite catalog JSON into a canonical pretty-printed form after real compile validation
- `snapshot-normalize <input> [output]` to rewrite contract-shaped runtime snapshots into the same canonical pretty-printed form
- `asset-bundle-check <manifest>` to validate a `switchyard.asset-bundle-cli` asset bundle manifest, resolve its referenced catalog and snapshot assets, and run the CLI validation flow across the bundle

Run it locally with:

```bash
cargo run -p switchyard-cli -- catalog-check fixtures/contracts/program.valid.json
cargo run -p switchyard-cli -- catalog-summary fixtures/contracts/program.valid.json
cargo run -p switchyard-cli -- snapshot-summary fixtures/contracts/snapshot.valid.json
cargo run -p switchyard-cli -- snapshot-check fixtures/contracts/program.valid.json fixtures/contracts/snapshot.valid.json
cargo run -p switchyard-cli -- asset-bundle-summary fixtures/contracts/asset-bundle.valid.json
cargo run -p switchyard-cli -- catalog-normalize fixtures/contracts/program.valid.json
cargo run -p switchyard-cli -- snapshot-normalize fixtures/contracts/snapshot.valid.json
cargo run -p switchyard-cli -- asset-bundle-check fixtures/contracts/asset-bundle.valid.json
```

In the browser showcase, the CLI handoff panel now supports copying or downloading both the
current catalog and a CLI-compatible runtime snapshot, then validating the pair with
`switchyard-cli snapshot-check`.

For multi-file handoff flows, the same exported assets can be wrapped in a
`switchyard.asset-bundle-cli` manifest, inspected with
`asset-bundle-summary <manifest>`, and validated end-to-end with
`asset-bundle-check <manifest>`.

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
- Per-task `MindId` scheduling with `change_mind` and host-controlled active-mind gating
- Predicate-based structured branching as an atomic choose-child-and-join primitive
- Explicit `sync_children` authoring lowered into enum-backed spawn/join ops
- Explicit `race_children` authoring lowered into the enum-backed two-branch race primitive
- Bounded `repeat_count` authoring convenience lowered into enum-backed spawn/join ops
- `repeat_until_predicate` as an enum-backed repeat primitive with deterministic same-tick re-entry control
- `wait_signal_or_ticks` as an enum-backed timeout primitive for signal-or-deadline waits
- `wait_until_tick`, `wait_signal_until_tick`, `timeout_until_tick`, and `race_children_until_tick` as authored absolute-deadline counterparts over the existing absolute `until_tick` wait states
- `timeout_ticks` as an enum-backed child deadline combinator that cancels overdue child routines
- `race_children_or_ticks` as an enum-backed timeout-race combinator that resolves on the first child winner or cancels both children at the deadline
- `join_any_children` as a first-finished-child barrier for spawned child routines
- External host calls with a fixed four-operand payload for gameplay commands such as projectile spawns and debug prints
- Snapshot export/import and plain-data task inspection
- Fixed-capacity `ProgramBuilder` for `no_std` authoring
- Alloc-backed `OwnedProgram` for dynamic authoring when `alloc` is available
- Contract-shaped `ProgramCatalogDocument` and `OwnedProgramCatalog` for runtime-editable script compilation
- Optional `serde` feature for snapshot JSON round-trip and program export/import
- Explicit `TraceEvent` / `TraceSink` runtime hooks
- `switchyard-debug::TraceLog` for reusable event capture and rendering
- Browser showcase for orchestrated encounter stepping, a representative shootemup boss-phase preset, a multi-mind handoff preset, visual script editing, and snapshot/trace inspection
- Native `shootemup` example showing host-call projectile bursts and boss-phase orchestration from a console host
- Native `director_handoff` example showing host-driven mind handoff plus snapshot restore outside the browser showcase
- `switchyard-cli` for deterministic catalog compile checks plus catalog/snapshot JSON summaries
- `switchyard-cli snapshot-check` for catalog-versus-snapshot compatibility validation in save/load and editor export pipelines
- `switchyard-cli asset-bundle-check` plus `asset-bundle-summary` for validating and inspecting whole catalog/snapshot asset sets in CI and tooling
- `switchyard-cli` normalization commands for canonical catalog/snapshot output in CI and editor save flows
- Browser-to-CLI export flow for handing live-authored catalogs and runtime snapshots into `switchyard-cli`
- Contract schemas, valid/invalid fixtures, CI, GitHub Pages deployment, and prompt-pack loop automation

## Remaining expansion areas

- More sample integrations beyond the current browser presets and native shootemup/director handoff examples
- Richer asset-pipeline tooling on top of the new catalog/snapshot CLI and asset-bundle manifest flow

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
