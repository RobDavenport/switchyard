#![cfg(feature = "serde")]

use switchyard_core::{
    ActionId, Host, MindId, Op, Program, ProgramCatalog, ProgramId, Runtime, RuntimeSnapshot,
    SignalId, TaskId,
};

struct TestHost {
    actions: std::vec::Vec<(u32, u16)>,
    active_minds: std::vec::Vec<MindId>,
}

impl Default for TestHost {
    fn default() -> Self {
        Self { actions: vec![], active_minds: vec![MindId(1)] }
    }
}

impl Host for TestHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push((task.0, action.0));
    }

    fn is_mind_active(&mut self, mind: MindId) -> bool {
        self.active_minds.contains(&mind)
    }
}

#[test]
fn runtime_snapshot_round_trips_through_serde_json() {
    const MAIN: [Op; 4] = [
        Op::ChangeMind(MindId(2)),
        Op::WaitSignal(SignalId(7)),
        Op::Action(ActionId(1)),
        Op::Succeed,
    ];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost { active_minds: vec![MindId(1), MindId(2)], ..Default::default() };
    let task = runtime.spawn(ProgramId(1)).expect("spawn root");

    runtime.tick(&mut host).expect("wait for signal");
    let json = serde_json::to_string_pretty(&runtime.snapshot()).expect("serialize snapshot");
    let restored_snapshot: RuntimeSnapshot<8, 4> =
        serde_json::from_str(&json).expect("deserialize snapshot");
    assert_eq!(restored_snapshot.tasks[0].expect("task present").mind_id, MindId(2));

    let mut restored = Runtime::from_snapshot(catalog, restored_snapshot);
    restored.emit_signal(SignalId(7)).expect("emit signal");
    restored.tick(&mut host).expect("resume from restored snapshot");

    assert_eq!(host.actions, vec![(task.0, 1)]);
}

#[test]
fn runtime_snapshot_round_trips_signal_or_ticks_wait_reason() {
    const MAIN: [Op; 3] =
        [Op::WaitSignalOrTicks(SignalId(7), 2), Op::Action(ActionId(2)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    runtime.spawn(ProgramId(1)).expect("spawn root");
    runtime.tick(&mut host).expect("enter signal-or-timeout wait");
    let json = serde_json::to_string_pretty(&runtime.snapshot()).expect("serialize snapshot");
    let restored_snapshot: RuntimeSnapshot<8, 4> =
        serde_json::from_str(&json).expect("deserialize snapshot");

    match restored_snapshot.tasks[0].expect("task present").wait {
        switchyard_core::WaitReason::SignalOrTicks { signal, until_tick } => {
            assert_eq!(signal, SignalId(7));
            assert_eq!(until_tick, 3);
        }
        other => panic!("unexpected wait reason: {other:?}"),
    }
}

#[test]
fn runtime_snapshot_round_trips_timeout_wait_reason() {
    const PARENT: [Op; 3] =
        [Op::TimeoutTicks(2, ProgramId(2)), Op::Action(ActionId(9)), Op::Succeed];
    const CHILD: [Op; 2] = [Op::WaitSignal(SignalId(7)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    runtime.spawn(ProgramId(1)).expect("spawn root");
    runtime.tick(&mut host).expect("enter timeout wait");
    let json = serde_json::to_string_pretty(&runtime.snapshot()).expect("serialize snapshot");
    let restored_snapshot: RuntimeSnapshot<8, 4> =
        serde_json::from_str(&json).expect("deserialize snapshot");

    match restored_snapshot.tasks[0].expect("parent present").wait {
        switchyard_core::WaitReason::Timeout { child, until_tick } => {
            assert_eq!(child, TaskId(2));
            assert_eq!(until_tick, 3);
        }
        other => panic!("unexpected wait reason: {other:?}"),
    }
}

#[test]
fn runtime_snapshot_round_trips_race_or_ticks_wait_reason() {
    const PARENT: [Op; 3] = [
        Op::RaceChildrenOrTicks(ProgramId(2), ProgramId(3), 2),
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const LEFT: [Op; 2] = [Op::WaitSignal(SignalId(7)), Op::Succeed];
    const RIGHT: [Op; 2] = [Op::WaitSignal(SignalId(8)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &LEFT),
        Program::new(ProgramId(3), &RIGHT),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    runtime.spawn(ProgramId(1)).expect("spawn root");
    runtime.tick(&mut host).expect("enter race-or-timeout wait");
    let json = serde_json::to_string_pretty(&runtime.snapshot()).expect("serialize snapshot");
    let restored_snapshot: RuntimeSnapshot<8, 4> =
        serde_json::from_str(&json).expect("deserialize snapshot");

    match restored_snapshot.tasks[0].expect("parent present").wait {
        switchyard_core::WaitReason::RaceOrTicks { left, right, until_tick } => {
            assert_eq!(left, TaskId(2));
            assert_eq!(right, TaskId(3));
            assert_eq!(until_tick, 3);
        }
        other => panic!("unexpected wait reason: {other:?}"),
    }
}
