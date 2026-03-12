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
