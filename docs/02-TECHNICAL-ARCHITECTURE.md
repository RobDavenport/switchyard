# 02-TECHNICAL-ARCHITECTURE

## System decomposition

### Core crate

A single `switchyard-core` crate owns IDs, programs, scheduler, and snapshots. Optional future crates can layer tooling on top without entering the hot path.

### Boundaries

The host controls time by calling `tick()`. The host is also the only source of signals, predicate readiness, active-mind scheduling, and side-effect execution via `Host::on_action` and `Host::on_call`.

### Data flow

Program catalog -> runtime spawn -> host-driven tick -> filter runnable tasks by active mind -> wake waiting tasks -> execute ops in stable order -> emit host actions or host calls -> snapshot or inspect state.

### Contracts

JSON Schema files describe the walking-skeleton program catalog and runtime snapshot fixtures. The current code treats these as documentation and fixture contracts rather than production parsing code.

### Storage strategy

Fixed-capacity arrays back active task slots and pending signals. Task IDs are monotonic, per-task `mind_id` membership is persisted in snapshots and inspection output, and no mandatory allocator is required.

### Integration points

Host integration is intentionally narrow:
- `Host::on_action` for effect emission
- `Host::on_call` for deterministic external gameplay commands
- `Host::query_ready` for explicit predicate wake-up
- `Host::is_mind_active` for host-controlled mind gating
- `emit_signal` for externally supplied event IDs

### Security and performance

No hidden I/O or threads. Ordering is explicit. Hot-path execution avoids strings and heap churn. The walking skeleton chooses clarity over maximum throughput.

### Rationale for stack choice

Rust stable keeps the core portable and testable. The repo intentionally avoids mandatory third-party runtime deps to keep offline bootstrap and no_std validation straightforward.
