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
| Authoring DSL | `crates/switchyard-core/tests/program_builder.rs::{builder_authors_a_runnable_program_in_order,builder_rejects_ops_past_fixed_capacity}` | `switchyard-core` | green |
| Trace hook ordering | `crates/switchyard-core/tests/trace.rs::{trace_reports_signal_wait_wake_action_and_finish_in_order,trace_reports_race_winner_and_loser_cancellation_in_order}` | `switchyard-core` | green |
