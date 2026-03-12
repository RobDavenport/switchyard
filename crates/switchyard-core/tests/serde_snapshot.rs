#![cfg(feature = "serde")]

use switchyard_core::{
    ActionId, Host, Op, Program, ProgramCatalog, ProgramId, Runtime, RuntimeSnapshot, SignalId,
    TaskId,
};

#[derive(Default)]
struct TestHost {
    actions: std::vec::Vec<(u32, u16)>,
}

impl Host for TestHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push((task.0, action.0));
    }
}

#[test]
fn runtime_snapshot_round_trips_through_serde_json() {
    const MAIN: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();
    let task = runtime.spawn(ProgramId(1)).expect("spawn root");

    runtime.tick(&mut host).expect("wait for signal");
    let json = serde_json::to_string_pretty(&runtime.snapshot()).expect("serialize snapshot");
    let restored_snapshot: RuntimeSnapshot<8, 4> =
        serde_json::from_str(&json).expect("deserialize snapshot");

    let mut restored = Runtime::from_snapshot(catalog, restored_snapshot);
    restored.emit_signal(SignalId(7)).expect("emit signal");
    restored.tick(&mut host).expect("resume from restored snapshot");

    assert_eq!(host.actions, vec![(task.0, 1)]);
}
