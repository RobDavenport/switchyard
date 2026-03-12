#![cfg(feature = "alloc")]

use switchyard_core::{
    ActionId, CompileError, Host, HostCall, HostCallId, MindId, OpDocument, ProgramCatalogDocument,
    ProgramDocument, ProgramId, Runtime, SignalId, TaskId,
};

struct TestHost {
    actions: Vec<(u32, u16)>,
    calls: Vec<(u32, u16, [i32; 4])>,
    active_minds: Vec<MindId>,
    ready_predicates: Vec<switchyard_core::PredicateId>,
}

impl Default for TestHost {
    fn default() -> Self {
        Self {
            actions: vec![],
            calls: vec![],
            active_minds: vec![MindId(1)],
            ready_predicates: vec![],
        }
    }
}

impl Host for TestHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push((task.0, action.0));
    }

    fn on_call(&mut self, task: TaskId, call: HostCall) {
        self.calls.push((task.0, call.id.0, call.args));
    }

    fn is_mind_active(&mut self, mind: MindId) -> bool {
        self.active_minds.contains(&mind)
    }

    fn query_ready(&mut self, predicate: switchyard_core::PredicateId) -> bool {
        self.ready_predicates.contains(&predicate)
    }
}

#[test]
fn catalog_document_compiles_to_runnable_runtime_programs() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::Spawn { program: ProgramId(2) },
                    OpDocument::JoinChildren,
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("spawn child");
        runtime.emit_signal(SignalId(7)).expect("queue signal");
        runtime.tick(&mut host).expect("resolve child and join");

        assert_eq!(host.actions, vec![(2, 1), (1, 9)]);
    });
}

#[test]
fn catalog_document_rejects_duplicate_program_ids() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument { id: ProgramId(1), ops: vec![OpDocument::Succeed] },
            ProgramDocument { id: ProgramId(1), ops: vec![OpDocument::Succeed] },
        ],
    };

    assert_eq!(document.compile(), Err(CompileError::DuplicateProgramId(ProgramId(1))));
}

#[test]
fn catalog_document_rejects_missing_program_reference() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![OpDocument::Spawn { program: ProgramId(2) }, OpDocument::Succeed],
        }],
    };

    assert_eq!(
        document.compile(),
        Err(CompileError::UnknownProgramReference { owner: ProgramId(1), target: ProgramId(2) })
    );
}

#[test]
fn catalog_document_rejects_missing_branch_target_reference() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![
                OpDocument::BranchPredicate {
                    predicate: switchyard_core::PredicateId(5),
                    if_true: ProgramId(2),
                    if_false: ProgramId(3),
                },
                OpDocument::Succeed,
            ],
        }],
    };

    assert_eq!(
        document.compile(),
        Err(CompileError::UnknownProgramReference { owner: ProgramId(1), target: ProgramId(2) })
    );
}

#[test]
fn catalog_document_call_op_invokes_host() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![
                OpDocument::Call { call: HostCallId(7), arg0: 10, arg1: 20, arg2: 30, arg3: 40 },
                OpDocument::Succeed,
            ],
        }],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<4, 2> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        let task = runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("emit host call");

        assert_eq!(host.calls, vec![(task.0, 7, [10, 20, 30, 40])]);
    });
}

#[test]
fn catalog_document_wait_signal_or_ticks_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![
                OpDocument::WaitSignalOrTicks { signal: SignalId(7), ticks: 2 },
                OpDocument::Action { action: ActionId(4) },
                OpDocument::Succeed,
            ],
        }],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<4, 2> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("enter wait");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("timeout");

        assert_eq!(host.actions, vec![(1, 4)]);
    });
}

#[test]
fn catalog_document_wait_until_tick_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![
                OpDocument::WaitUntilTick { until_tick: 3 },
                OpDocument::Action { action: ActionId(4) },
                OpDocument::Succeed,
            ],
        }],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<4, 2> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("enter wait");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("absolute deadline");

        assert_eq!(host.actions, vec![(1, 4)]);
    });
}

#[test]
fn catalog_document_wait_signal_until_tick_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![
                OpDocument::WaitSignalUntilTick { signal: SignalId(7), until_tick: 3 },
                OpDocument::Action { action: ActionId(4) },
                OpDocument::Succeed,
            ],
        }],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<4, 2> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("enter wait");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("absolute deadline");

        assert_eq!(host.actions, vec![(1, 4)]);
    });
}

#[test]
fn catalog_document_timeout_until_tick_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::TimeoutUntilTick { until_tick: 3, program: ProgramId(2) },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("start timeout");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("deadline");

        assert_eq!(host.actions, vec![(1, 9)]);
        assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, switchyard_core::Outcome::Cancelled);
    });
}

#[test]
fn catalog_document_race_children_until_tick_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::RaceChildrenUntilTick {
                        left: ProgramId(2),
                        right: ProgramId(3),
                        until_tick: 3,
                    },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(3),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(8) },
                    OpDocument::Action { action: ActionId(2) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("spawn children");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("deadline");

        assert_eq!(host.actions, vec![(1, 9)]);
        assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, switchyard_core::Outcome::Cancelled);
        assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, switchyard_core::Outcome::Cancelled);
    });
}

#[test]
fn catalog_document_timeout_ticks_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::TimeoutTicks { ticks: 2, program: ProgramId(2) },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("start timeout");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("timeout");

        assert_eq!(host.actions, vec![(1, 9)]);
        assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, switchyard_core::Outcome::Cancelled);
    });
}

#[test]
fn catalog_document_race_children_or_ticks_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::RaceChildrenOrTicks {
                        left: ProgramId(2),
                        right: ProgramId(3),
                        ticks: 2,
                    },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(3),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(8) },
                    OpDocument::Action { action: ActionId(2) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("spawn race children");
        runtime.tick(&mut host).expect("countdown");
        runtime.tick(&mut host).expect("deadline resolves parent");

        assert_eq!(host.actions, vec![(1, 9)]);
        assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, switchyard_core::Outcome::Cancelled);
        assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, switchyard_core::Outcome::Cancelled);
    });
}

#[test]
fn catalog_document_change_mind_compiles_to_runnable_runtime_program() {
    let document = ProgramCatalogDocument {
        programs: vec![ProgramDocument {
            id: ProgramId(1),
            ops: vec![
                OpDocument::ChangeMind { mind: MindId(2) },
                OpDocument::Action { action: ActionId(8) },
                OpDocument::Succeed,
            ],
        }],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<4, 2> = Runtime::new(borrowed);
        let mut host = TestHost { active_minds: vec![MindId(1)], ..Default::default() };

        let task = runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("change mind");
        assert!(host.actions.is_empty());
        assert_eq!(runtime.task(task).unwrap().mind_id, MindId(2));

        host.active_minds.push(MindId(2));
        runtime.tick(&mut host).expect("resume on new mind");
        assert_eq!(host.actions, vec![(task.0, 8)]);
    });
}

#[test]
fn catalog_document_repeat_until_predicate_compiles_to_repeated_child_runs() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::RepeatUntilPredicate {
                        predicate: switchyard_core::PredicateId(5),
                        program: ProgramId(2),
                    },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![OpDocument::Action { action: ActionId(1) }, OpDocument::Succeed],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("run first repeat iteration");
        runtime.tick(&mut host).expect("run second repeat iteration");
        host.ready_predicates.push(switchyard_core::PredicateId(5));
        runtime.tick(&mut host).expect("finish once predicate becomes ready");

        assert_eq!(
            host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(),
            vec![1, 1, 9]
        );
    });
}

#[test]
fn catalog_document_sync_children_compiles_to_spawn_and_join() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::SyncChildren { left: ProgramId(2), right: ProgramId(3) },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![OpDocument::Action { action: ActionId(1) }, OpDocument::Succeed],
            },
            ProgramDocument {
                id: ProgramId(3),
                ops: vec![OpDocument::Action { action: ActionId(2) }, OpDocument::Succeed],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("run sync children and continue");

        assert_eq!(
            host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(),
            vec![1, 2, 9]
        );
    });
}

#[test]
fn catalog_document_repeat_count_compiles_to_repeated_child_runs() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::RepeatCount { count: 3, program: ProgramId(2) },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![OpDocument::Action { action: ActionId(1) }, OpDocument::Succeed],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<16, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("run repeated children");

        assert_eq!(
            host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(),
            vec![1, 1, 1, 9]
        );
    });
}

#[test]
fn catalog_document_repeat_count_zero_is_a_noop() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::RepeatCount { count: 0, program: ProgramId(2) },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![OpDocument::Action { action: ActionId(1) }, OpDocument::Succeed],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("skip zero repeat");

        assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![9]);
    });
}

#[test]
fn catalog_document_join_any_compiles_to_resume_on_first_completed_child() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::Spawn { program: ProgramId(2) },
                    OpDocument::Spawn { program: ProgramId(3) },
                    OpDocument::JoinAnyChildren,
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(3),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(8) },
                    OpDocument::Action { action: ActionId(2) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("spawn children");
        runtime.emit_signal(SignalId(8)).expect("queue fast signal");
        runtime.tick(&mut host).expect("resolve first child and parent");

        assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![2, 9]);
    });
}

#[test]
fn catalog_document_race_children_compiles_to_existing_race_semantics() {
    let document = ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: ProgramId(1),
                ops: vec![
                    OpDocument::RaceChildren { left: ProgramId(2), right: ProgramId(3) },
                    OpDocument::Action { action: ActionId(9) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(2),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(7) },
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ProgramId(3),
                ops: vec![
                    OpDocument::WaitSignal { signal: SignalId(8) },
                    OpDocument::Action { action: ActionId(2) },
                    OpDocument::Succeed,
                ],
            },
        ],
    };

    let catalog = document.compile().expect("compile catalog");
    catalog.with_catalog(|borrowed| {
        let mut runtime: Runtime<8, 4> = Runtime::new(borrowed);
        let mut host = TestHost::default();

        runtime.spawn(ProgramId(1)).expect("spawn root");
        runtime.tick(&mut host).expect("spawn race children");
        runtime.emit_signal(SignalId(7)).expect("queue winning signal");
        runtime.tick(&mut host).expect("resolve winner and parent");

        assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 9]);
    });
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_through_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize contract-shaped document");

    assert_eq!(
        document.programs[0].ops,
        vec![
            OpDocument::WaitSignal { signal: SignalId(7) },
            OpDocument::Action { action: ActionId(1) },
            OpDocument::Succeed,
        ]
    );

    let encoded = serde_json::to_string(&document).expect("serialize contract-shaped document");
    assert!(encoded.contains("\"wait_signal\""));
    assert!(encoded.contains("\"action\""));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_call_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "call", "call": 7, "arg0": 10, "arg1": 20, "arg2": 30, "arg3": 40 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize call contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::Call { call: HostCallId(7), arg0: 10, arg1: 20, arg2: 30, arg3: 40 }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize call contract-shaped document");
    assert!(encoded.contains("\"call\""));
    assert!(encoded.contains("\"arg0\":10"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_wait_signal_or_ticks_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_signal_or_ticks", "signal": 7, "ticks": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize signal-or-timeout contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::WaitSignalOrTicks { signal: SignalId(7), ticks: 2 }
    );

    let encoded = serde_json::to_string(&document)
        .expect("serialize signal-or-timeout contract-shaped document");
    assert!(encoded.contains("\"wait_signal_or_ticks\""));
    assert!(encoded.contains("\"signal\":7"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_wait_until_tick_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_until_tick", "until_tick": 3 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize absolute-wait contract-shaped document");

    assert_eq!(document.programs[0].ops[0], OpDocument::WaitUntilTick { until_tick: 3 });

    let encoded =
        serde_json::to_string(&document).expect("serialize absolute-wait contract-shaped document");
    assert!(encoded.contains("\"wait_until_tick\""));
    assert!(encoded.contains("\"until_tick\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_wait_signal_until_tick_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_signal_until_tick", "signal": 7, "until_tick": 3 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument = serde_json::from_str(json)
        .expect("deserialize absolute signal-wait contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::WaitSignalUntilTick { signal: SignalId(7), until_tick: 3 }
    );

    let encoded = serde_json::to_string(&document)
        .expect("serialize absolute signal-wait contract-shaped document");
    assert!(encoded.contains("\"wait_signal_until_tick\""));
    assert!(encoded.contains("\"until_tick\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_timeout_until_tick_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "timeout_until_tick", "until_tick": 3, "program": 2 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize absolute timeout contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::TimeoutUntilTick { until_tick: 3, program: ProgramId(2) }
    );

    let encoded = serde_json::to_string(&document)
        .expect("serialize absolute timeout contract-shaped document");
    assert!(encoded.contains("\"timeout_until_tick\""));
    assert!(encoded.contains("\"until_tick\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_race_children_until_tick_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "race_children_until_tick", "left": 2, "right": 3, "until_tick": 3 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize absolute race contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::RaceChildrenUntilTick {
            left: ProgramId(2),
            right: ProgramId(3),
            until_tick: 3,
        }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize absolute race contract-shaped document");
    assert!(encoded.contains("\"race_children_until_tick\""));
    assert!(encoded.contains("\"until_tick\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_timeout_ticks_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "timeout_ticks", "ticks": 2, "program": 2 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize timeout contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::TimeoutTicks { ticks: 2, program: ProgramId(2) }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize timeout contract-shaped document");
    assert!(encoded.contains("\"timeout_ticks\""));
    assert!(encoded.contains("\"ticks\":2"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_race_children_or_ticks_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "race_children_or_ticks", "left": 2, "right": 3, "ticks": 2 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize race-timeout contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::RaceChildrenOrTicks { left: ProgramId(2), right: ProgramId(3), ticks: 2 }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize race-timeout contract-shaped document");
    assert!(encoded.contains("\"race_children_or_ticks\""));
    assert!(encoded.contains("\"ticks\":2"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_change_mind_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "change_mind", "mind": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize change-mind contract-shaped document");

    assert_eq!(document.programs[0].ops[0], OpDocument::ChangeMind { mind: MindId(2) });

    let encoded =
        serde_json::to_string(&document).expect("serialize change-mind contract-shaped document");
    assert!(encoded.contains("\"change_mind\""));
    assert!(encoded.contains("\"mind\":2"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_sync_children_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "sync_children", "left": 2, "right": 3 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize sync contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::SyncChildren { left: ProgramId(2), right: ProgramId(3) }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize sync contract-shaped document");
    assert!(encoded.contains("\"sync_children\""));
    assert!(encoded.contains("\"left\":2"));
    assert!(encoded.contains("\"right\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_race_children_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "race_children", "left": 2, "right": 3 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize race contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::RaceChildren { left: ProgramId(2), right: ProgramId(3) }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize race contract-shaped document");
    assert!(encoded.contains("\"race_children\""));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_branch_predicate_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "branch_predicate", "predicate": 5, "if_true": 2, "if_false": 3 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize branch contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::BranchPredicate {
            predicate: switchyard_core::PredicateId(5),
            if_true: ProgramId(2),
            if_false: ProgramId(3),
        }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize branch contract-shaped document");
    assert!(encoded.contains("\"branch_predicate\""));
    assert!(encoded.contains("\"if_true\":2"));
    assert!(encoded.contains("\"if_false\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_repeat_count_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "repeat_count", "count": 3, "program": 2 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize repeat contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::RepeatCount { count: 3, program: ProgramId(2) }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize repeat contract-shaped document");
    assert!(encoded.contains("\"repeat_count\""));
    assert!(encoded.contains("\"count\":3"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_repeat_until_predicate_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "repeat_until_predicate", "predicate": 5, "program": 2 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize repeat-until contract-shaped document");

    assert_eq!(
        document.programs[0].ops[0],
        OpDocument::RepeatUntilPredicate {
            predicate: switchyard_core::PredicateId(5),
            program: ProgramId(2),
        }
    );

    let encoded =
        serde_json::to_string(&document).expect("serialize repeat-until contract-shaped document");
    assert!(encoded.contains("\"repeat_until_predicate\""));
    assert!(encoded.contains("\"predicate\":5"));
}

#[cfg(feature = "serde")]
#[test]
fn catalog_document_round_trips_join_any_children_contract_json() {
    let json = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "spawn", "program": 2 },
        { "op": "spawn", "program": 3 },
        { "op": "join_any_children" },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 1 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 2 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    let document: ProgramCatalogDocument =
        serde_json::from_str(json).expect("deserialize join-any contract-shaped document");

    assert_eq!(document.programs[0].ops[2], OpDocument::JoinAnyChildren);

    let encoded =
        serde_json::to_string(&document).expect("serialize join-any contract-shaped document");
    assert!(encoded.contains("\"join_any_children\""));
}
