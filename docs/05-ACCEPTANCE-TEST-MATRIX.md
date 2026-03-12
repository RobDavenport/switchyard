# 05-ACCEPTANCE-TEST-MATRIX

| Requirement | Test(s) / fixture(s) | Owner / module | Seed status |
|---|---|---|---|
| Deterministic tick execution | `crates/switchyard-core/tests/smoke.rs::wait_ticks_then_action` | `switchyard-core` | green |
| External signal wake-up | `crates/switchyard-core/tests/smoke.rs::snapshot_restore_resumes_waiting_task` | `switchyard-core` | green |
| Predicate wake-up | `crates/switchyard-core/tests/smoke.rs::predicate_wait_blocks_until_host_ready` | `switchyard-core` | green |
| Join-all behavior | `crates/switchyard-core/tests/smoke.rs::spawn_then_join_is_ordered` | `switchyard-core` | green |
| Race semantics | `crates/switchyard-core/tests/smoke.rs::race_cancels_loser` | `switchyard-core` | green |
| Snapshot contract examples | `fixtures/contracts/snapshot.valid.json`, `snapshot.invalid.json`, `scripts/validate_contract_fixtures.py` | `contracts/` | green |
| Program catalog contract examples | `fixtures/contracts/program.valid.json`, `program.invalid.json`, `scripts/validate_contract_fixtures.py` | `contracts/` | green |
| Authoring DSL | `crates/switchyard-core/tests/program_builder.rs::{builder_authors_a_runnable_program_in_order,builder_rejects_ops_past_fixed_capacity}`, `crates/switchyard-core/tests/owned_program.rs::{owned_program_authors_a_runnable_program,owned_program_clear_reuses_builder_storage}` | `switchyard-core` | green |
| Serde snapshot and program round-trip | `crates/switchyard-core/tests/serde_snapshot.rs::runtime_snapshot_round_trips_through_serde_json`, `crates/switchyard-core/tests/serde_program.rs::owned_program_round_trips_through_serde_json` | `switchyard-core` | green |
| Trace hook ordering | `crates/switchyard-core/tests/trace.rs::{trace_reports_signal_wait_wake_action_and_finish_in_order,trace_reports_race_winner_and_loser_cancellation_in_order}` | `switchyard-core` | green |
| Debug trace log tooling | `crates/switchyard-debug/tests/trace_log.rs::{trace_log_records_events_from_runtime,trace_log_clear_drops_prior_events}` | `switchyard-debug` | green |
| Prompt-pack loop | `scripts/test_run_prompt_pack.py` | `scripts/` | green |
| Browser WASM showcase | `demo-wasm/tests/showcase.rs::{showcase_view_exposes_waiting_signal_and_trace_after_first_tick,showcase_snapshot_restore_can_flip_race_outcome}`, `demo-wasm/www/index.html`, `.github/workflows/pages.yml` | `demo-wasm` | green |
