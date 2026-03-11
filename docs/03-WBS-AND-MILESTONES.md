# 03-WBS-AND-MILESTONES

| Work package | Dependencies | Deliverables | Done criteria |
|---|---|---|---|
| WP-01 repo bootstrap | none | Workspace root, docs set, CI workflow, taskboard, contract validation script | Top-level commands and file layout are coherent and documented |
| WP-02 core model | WP-01 | IDs, op model, program catalog, snapshot types | Typed domain model exists with compile-ready Rust files and tests |
| WP-03 scheduler skeleton | WP-02 | Deterministic tick loop, wait states, spawn/join/race/cancel behavior | Scenario tests pass for waits, joins, races, and predicates |
| WP-04 inspection + persistence | WP-03 | Snapshot export/import, task inspection helpers, fixtures | Snapshot restore scenario passes and fixtures validate |
| WP-05 examples + docs | WP-03, WP-04 | Example cutscene, README workflow, acceptance matrix | Docs match code and example behavior |
| WP-06 CI + release hygiene | WP-01..WP-05 | fmt/clippy/test/no-default-features/docs checks | CI script covers declared developer workflow |
