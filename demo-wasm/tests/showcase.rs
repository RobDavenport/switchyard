use switchyard_demo_wasm::ShowcaseState;

#[test]
fn showcase_view_exposes_waiting_signal_and_trace_after_first_tick() {
    let mut showcase = ShowcaseState::new();

    showcase.tick().expect("first tick");
    let view = showcase.view();

    assert_eq!(view.clock, 1);
    assert!(view.beat.contains("Camera crawls"));
    assert!(view.tasks.iter().any(
        |task| task.program_label == "main encounter" && task.wait.contains("player committed")
    ));
    assert!(view.trace.contains("task_waiting"));
}

#[test]
fn showcase_snapshot_restore_can_flip_race_outcome() {
    let mut showcase = ShowcaseState::new();

    showcase.tick().expect("intro tick");
    showcase.emit_signal(switchyard_core::SignalId(1)).expect("player committed");
    showcase.tick().expect("spawn children tick");
    showcase.emit_signal(switchyard_core::SignalId(2)).expect("scouts ready");
    showcase.tick().expect("scout clear tick");
    showcase.tick().expect("gate clear tick");
    showcase.set_boss_vulnerable(true);
    showcase.tick().expect("predicate and race tick");

    let snapshot = showcase.save_snapshot().expect("save snapshot");

    showcase.emit_signal(switchyard_core::SignalId(4)).expect("collapse branch");
    showcase.tick().expect("collapse resolves");
    assert!(showcase.view().actions.iter().any(|action| action.label.contains("escape route")));

    showcase.load_snapshot(&snapshot).expect("restore snapshot");
    showcase.emit_signal(switchyard_core::SignalId(3)).expect("boss spotted branch");
    showcase.tick().expect("boss intro resolves");

    let labels: Vec<String> =
        showcase.view().actions.into_iter().map(|action| action.label).collect();
    assert!(labels.iter().any(|label| label.contains("boss steps")));
    assert!(!labels.iter().any(|label| label.contains("escape route")));
}

#[test]
fn showcase_can_switch_to_shootemup_preset() {
    let mut showcase = ShowcaseState::new();

    showcase.load_preset("shootemup").expect("switch to shootemup preset");
    showcase.tick().expect("intro tick");

    let view = showcase.view();
    assert_eq!(view.preset, "shootemup");
    assert_eq!(view.title, "Shootemup Boss");
    assert!(view.beat.contains("boss frame drifts into the lane"));
    assert!(view
        .tasks
        .iter()
        .any(|task| task.program_label == "boss phase" && task.wait.contains("wave started")));
}

#[test]
fn showcase_shootemup_preset_emits_projectile_host_calls() {
    let mut showcase = ShowcaseState::new();

    showcase.load_preset("shootemup").expect("switch to shootemup preset");
    showcase.tick().expect("intro tick");
    showcase.emit_signal(switchyard_core::SignalId(1)).expect("engage boss phase");
    showcase.tick().expect("spawn pattern tasks");
    showcase.tick().expect("pattern timeline tick");

    let view = showcase.view();
    assert!(view.actions.iter().any(|action| action.action_id == 1));
    assert!(view.actions.iter().any(|action| action.label.contains("Spawn projectile")));
}

#[test]
fn showcase_shootemup_snapshot_restore_preserves_pattern_timeline() {
    let mut showcase = ShowcaseState::new();

    showcase.load_preset("shootemup").expect("switch to shootemup preset");
    showcase.tick().expect("intro tick");
    showcase.emit_signal(switchyard_core::SignalId(1)).expect("engage boss phase");
    showcase.tick().expect("spawn pattern tasks");
    showcase.tick().expect("pattern timeline tick");
    showcase.tick().expect("pattern timeline tick");
    showcase.tick().expect("pattern window resolves");
    showcase.set_boss_vulnerable(true);
    showcase.tick().expect("enter branch race");

    let snapshot = showcase.save_snapshot().expect("save shootemup snapshot");

    showcase.tick().expect("advance branch race");
    showcase.tick().expect("let enrage branch win");
    assert!(showcase.view().actions.iter().any(|action| action.label.contains("hits enrage")));

    showcase.load_snapshot(&snapshot).expect("restore shootemup snapshot");
    let restored_view = showcase.view();
    assert_eq!(restored_view.preset, "shootemup");
    assert!(restored_view.actions.iter().any(|action| action.label.contains("Spawn projectile")));
    assert!(!restored_view.actions.iter().any(|action| action.label.contains("hits enrage")));

    showcase.emit_signal(switchyard_core::SignalId(3)).expect("break boss core");
    showcase.tick().expect("resolve core break branch");

    let labels: Vec<String> =
        showcase.view().actions.into_iter().map(|action| action.label).collect();
    assert!(labels.iter().any(|label| label.contains("cracks the exposed core")));
    assert!(!labels.iter().any(|label| label.contains("hits enrage")));
}

#[test]
fn showcase_can_switch_to_multimind_preset() {
    let mut showcase = ShowcaseState::new();

    showcase.load_preset("multimind").expect("switch to multi-mind preset");
    showcase.tick().expect("intro tick");

    let view = showcase.view();
    assert_eq!(view.preset, "multimind");
    assert_eq!(view.title, "Mind the Gap");
    assert!(view.beat.contains("director marks the next scene"));
    assert!(view
        .tasks
        .iter()
        .any(|task| task.program_label == "director track" && task.wait.contains("director cue")));
}

#[test]
fn showcase_multimind_preset_blocks_until_named_mind_becomes_active() {
    let mut showcase = ShowcaseState::new();

    showcase.load_preset("multimind").expect("switch to multi-mind preset");
    showcase.tick().expect("intro tick");
    showcase.emit_signal(switchyard_core::SignalId(1)).expect("director cue");
    showcase.tick().expect("handoff to gameplay mind");

    let parked_view = showcase.view();
    assert_eq!(parked_view.active_mind, 1);
    assert!(parked_view.beat.contains("gameplay mind"));
    assert!(parked_view
        .actions
        .iter()
        .all(|action| !action.label.contains("gameplay mind takes control")));
    assert!(parked_view
        .tasks
        .iter()
        .any(|task| task.program_label == "director track" && task.mind == 2));

    showcase.set_active_mind(switchyard_core::MindId(2));
    showcase.tick().expect("resume on gameplay mind");

    let resumed_view = showcase.view();
    assert_eq!(resumed_view.active_mind, 2);
    assert!(resumed_view
        .actions
        .iter()
        .any(|action| action.label.contains("gameplay mind takes control")));
}

#[test]
fn showcase_snapshot_restore_preserves_multimind_wait_state() {
    let mut showcase = ShowcaseState::new();

    showcase.load_preset("multimind").expect("switch to multi-mind preset");
    showcase.tick().expect("intro tick");
    showcase.emit_signal(switchyard_core::SignalId(1)).expect("director cue");
    showcase.tick().expect("handoff to gameplay mind");

    let snapshot = showcase.save_snapshot().expect("save multi-mind snapshot");

    showcase.set_active_mind(switchyard_core::MindId(2));
    showcase.tick().expect("resume on gameplay mind");
    assert!(showcase
        .view()
        .actions
        .iter()
        .any(|action| action.label.contains("gameplay mind takes control")));

    showcase.load_snapshot(&snapshot).expect("restore multi-mind snapshot");

    let restored_view = showcase.view();
    assert_eq!(restored_view.preset, "multimind");
    assert_eq!(restored_view.active_mind, 1);
    assert!(restored_view.beat.contains("gameplay mind"));
    assert!(restored_view
        .actions
        .iter()
        .all(|action| !action.label.contains("gameplay mind takes control")));
    assert!(restored_view
        .tasks
        .iter()
        .any(|task| task.program_label == "director track" && task.mind == 2));

    showcase.set_active_mind(switchyard_core::MindId(2));
    showcase.tick().expect("resume restored gameplay mind");
    assert!(showcase
        .view()
        .actions
        .iter()
        .any(|action| action.label.contains("gameplay mind takes control")));
}

#[test]
fn showcase_can_load_runtime_script_document() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load runtime script");
    showcase.tick().expect("run edited script");

    let exported = showcase.script_json().expect("export current script");
    let exported_json: serde_json::Value =
        serde_json::from_str(&exported).expect("parse exported script json");
    let view = showcase.view();

    assert_eq!(exported_json["programs"][0]["ops"][0]["action"], 42);
    assert!(view.actions.iter().any(|action| action.action_id == 42));
}

#[test]
fn showcase_can_export_cli_compatible_catalog_document() {
    let showcase = ShowcaseState::new();

    let exported = showcase.export_cli_catalog_json().expect("export cli catalog");
    let exported_json: serde_json::Value =
        serde_json::from_str(&exported).expect("parse cli catalog json");

    assert!(exported_json.get("programs").is_some());
    assert_eq!(exported_json["programs"][0]["id"], 1);
    assert_eq!(exported_json["programs"][0]["ops"][0]["op"], "action");
}

#[test]
fn showcase_can_export_cli_compatible_runtime_snapshot() {
    let mut showcase = ShowcaseState::new();

    showcase.tick().expect("first tick");

    let exported =
        showcase.export_cli_runtime_snapshot_json().expect("export cli runtime snapshot");
    let exported_json: serde_json::Value =
        serde_json::from_str(&exported).expect("parse cli runtime snapshot json");

    assert!(exported_json.get("clock").is_some());
    assert!(exported_json.get("tasks").is_some());
    assert!(exported_json.get("pending_signals").is_some());
    assert!(exported_json.get("preset").is_none());
    assert_eq!(exported_json["clock"], 1);
    assert_eq!(exported_json["tasks"][0]["outcome"], "running");
    assert_eq!(exported_json["tasks"][0]["wait"]["kind"], "signal");
}

#[test]
fn showcase_runtime_script_can_emit_host_call() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "call", "call": 1, "arg0": 3, "arg1": 120, "arg2": 64, "arg3": 9 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load runtime host call script");
    showcase.tick().expect("emit host call");

    let view = showcase.view();
    assert!(view.actions.iter().any(|action| action.label.contains("Spawn projectile")));
    assert!(view.actions.iter().any(|action| action.action_id == 1));
}

#[test]
fn showcase_runtime_script_can_wait_for_signal_or_timeout() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_signal_or_ticks", "signal": 7, "ticks": 2 },
        { "op": "action", "action": 44 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load signal-or-timeout runtime script");
    showcase.tick().expect("enter wait");
    showcase.tick().expect("countdown");
    showcase.tick().expect("timeout resumes");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![44]);
}

#[test]
fn showcase_runtime_script_can_wait_until_tick() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_until_tick", "until_tick": 3 },
        { "op": "action", "action": 47 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load wait-until runtime script");
    showcase.tick().expect("advance toward deadline");
    showcase.tick().expect("advance toward deadline");
    showcase.tick().expect("resume at absolute deadline");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![47]);
}

#[test]
fn showcase_runtime_script_can_wait_for_signal_until_tick() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_signal_until_tick", "signal": 7, "until_tick": 3 },
        { "op": "action", "action": 48 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load signal-until runtime script");
    showcase.tick().expect("enter signal-until wait");
    showcase.tick().expect("advance toward deadline");
    showcase.tick().expect("resume on absolute deadline");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![48]);
}

#[test]
fn showcase_runtime_script_can_timeout_child_program() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "timeout_ticks", "ticks": 2, "program": 2 },
        { "op": "action", "action": 45 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load child-timeout runtime script");
    showcase.tick().expect("start timeout child");
    showcase.tick().expect("countdown");
    showcase.tick().expect("timeout child resumes parent");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![45]);
}

#[test]
fn showcase_runtime_script_can_timeout_child_until_tick() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "timeout_until_tick", "until_tick": 3, "program": 2 },
        { "op": "action", "action": 49 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load child-until runtime script");
    showcase.tick().expect("start absolute child timeout");
    showcase.tick().expect("advance toward deadline");
    showcase.tick().expect("absolute deadline resumes parent");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![49]);
}

#[test]
fn showcase_runtime_script_can_race_children_or_timeout() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "race_children_or_ticks", "left": 2, "right": 3, "ticks": 2 },
        { "op": "action", "action": 46 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "wait_signal", "signal": 8 },
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load race-or-timeout runtime script");
    showcase.tick().expect("spawn race-timeout children");
    showcase.tick().expect("countdown");
    showcase.tick().expect("timeout resolves race-timeout");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![46]);
}

#[test]
fn showcase_runtime_script_can_race_children_until_tick() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "race_children_until_tick", "left": 2, "right": 3, "until_tick": 3 },
        { "op": "action", "action": 50 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "wait_signal", "signal": 8 },
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load race-until runtime script");
    showcase.tick().expect("spawn absolute race children");
    showcase.tick().expect("advance toward deadline");
    showcase.tick().expect("absolute deadline resolves race");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![50]);
}

#[test]
fn showcase_runtime_script_can_change_mind() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "change_mind", "mind": 2 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load runtime change-mind script");
    showcase.tick().expect("change mind");

    let view = showcase.view();
    assert_eq!(view.active_mind, 1);
    assert!(view.actions.is_empty());
    assert_eq!(view.tasks[0].mind, 2);

    showcase.set_active_mind(switchyard_core::MindId(2));
    showcase.tick().expect("resume on active mind");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![41]);
}

#[test]
fn showcase_runtime_script_can_sync_children() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "sync_children", "left": 2, "right": 3 },
        { "op": "action", "action": 9 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load runtime sync script");
    showcase.tick().expect("run sync script");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![41, 42, 9]);
}

#[test]
fn showcase_runtime_script_can_race_children() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "race_children", "left": 2, "right": 3 },
        { "op": "action", "action": 9 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "wait_signal", "signal": 8 },
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load runtime race script");
    showcase.tick().expect("spawn race");
    showcase.emit_signal(switchyard_core::SignalId(7)).expect("signal winner");
    showcase.tick().expect("resolve race");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![41, 9]);
}

#[test]
fn showcase_runtime_script_can_branch_on_predicate() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "branch_predicate", "predicate": 1, "if_true": 2, "if_false": 3 },
        { "op": "action", "action": 9 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load branching runtime script");
    showcase.set_boss_vulnerable(true);
    showcase.tick().expect("run branching script");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![41, 9]);
}

#[test]
fn showcase_runtime_script_can_repeat_child_program() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "repeat_count", "count": 2, "program": 2 },
        { "op": "action", "action": 9 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load repeat runtime script");
    showcase.tick().expect("run repeat script");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![41, 41, 9]);
}

#[test]
fn showcase_runtime_script_can_repeat_until_predicate() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "repeat_until_predicate", "predicate": 1, "program": 2 },
        { "op": "action", "action": 9 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load repeat-until runtime script");
    showcase.tick().expect("run first repeat iteration");
    showcase.tick().expect("run second repeat iteration");
    showcase.set_boss_vulnerable(true);
    showcase.tick().expect("finish repeat-until script");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![41, 41, 9]);
}

#[test]
fn showcase_runtime_script_can_join_any_child() {
    let mut showcase = ShowcaseState::new();
    let script = r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "spawn", "program": 2 },
        { "op": "spawn", "program": 3 },
        { "op": "join_any_children" },
        { "op": "action", "action": 9 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 2,
      "ops": [
        { "op": "wait_signal", "signal": 7 },
        { "op": "action", "action": 41 },
        { "op": "succeed" }
      ]
    },
    {
      "id": 3,
      "ops": [
        { "op": "wait_signal", "signal": 8 },
        { "op": "action", "action": 42 },
        { "op": "succeed" }
      ]
    }
  ]
}"#;

    showcase.load_script(script).expect("load join-any runtime script");
    showcase.tick().expect("spawn children");
    showcase.emit_signal(switchyard_core::SignalId(8)).expect("signal fast child");
    showcase.tick().expect("resume parent from first child");

    let actions: Vec<u16> =
        showcase.view().actions.into_iter().map(|action| action.action_id).collect();
    assert_eq!(actions, vec![42, 9]);
}
