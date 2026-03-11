# 06-RISK-REGISTER

| Risk | Impact | Mitigation / descoping plan |
|---|---|---|
| Language-project scope creep | The crate becomes a parser/tooling project before core runtime semantics are proven. | Keep v0 focused on scheduler plus op model. Any DSL work must follow proven tests and remain optional. |
| Cancellation bugs | Incorrect teardown semantics can make save/load and race behavior unreliable. | Keep cancellation rules explicit, add regression tests per bug, and avoid hidden async machinery. |
| Capacity model surprises | Users may assume terminal task slots are recycled or allocations are available. | Document fixed-capacity behavior clearly and decide slot-reuse policy in a milestone with tests. |
| Schema drift | Docs and fixtures can diverge from code behavior. | Treat fixture validation and acceptance matrix updates as part of done criteria. |
