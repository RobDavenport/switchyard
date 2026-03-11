# 01-PRD

## Product

**switchyard**  
Deterministic, no_std structured-concurrency behavior runtime for game logic and orchestration.

## Users

Engine authors, gameplay programmers, simulation developers, custom runtime builders, and wasm-hosted game teams.

## Primary use cases

- Drive cutscenes, scripted encounters, tutorial flows, and quest logic without hand-written tick state machines.
- Express structured concurrency primitives such as wait, join-all, race, branch, and cancel-scope in a small runtime surface.
- Checkpoint and restore in-flight behavior for save/load, rollback, replay, or debugging.
- Embed the runtime inside wasm guests or native applications without hidden threads or global time sources.

## Core jobs to be done

- Schedule deterministic durational behavior from explicit host-driven ticks.
- Integrate with user-defined commands, events, and predicate queries through a narrow ABI.
- Inspect active tasks and wait reasons at runtime.
- Serialize and restore snapshots without patch-up closures or async internals.

## Must-have features

- Fixed-capacity deterministic scheduler in the core crate.
- Stable task IDs, explicit scopes, and explicit cancellation semantics.
- Bytecode-like op surface sufficient for wait, signal, predicate, spawn, join, race, succeed, and fail.
- Snapshot model and inspection APIs that expose active task state as plain data.
- Contract schemas and valid/invalid fixtures for programs and snapshots.

## Nice-to-have features

- Optional builder DSL once the execution semantics are proven.
- Optional serde-backed export/import after the snapshot contract settles.
- Optional tracing or debug-inspector crate as a sibling workspace member.

## Explicit non-goals

- General-purpose scripting language, parser, or dynamic object model.
- Automatic host API reflection or code generation.
- Hidden timers, threads, RNG, or I/O inside the core runtime.
- Framework-shaped ECS or actor assumptions in the public API.

## Launch scope

- One core crate with a working scheduler skeleton and representative tests.
- One example program that demonstrates orchestration of child routines.
- Boundary contracts for program catalogs and runtime snapshots.
- CI, docs, and agent prompt pack aligned with the walking skeleton state.

## Success criteria

- Representative behavior tests remain deterministic under repeated runs.
- Mid-behavior snapshot restore works without custom patch-up code.
- The core crate compiles with `--no-default-features`.
- The API surface stays small enough for non-framework adoption.
