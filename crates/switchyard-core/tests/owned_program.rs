#![cfg(feature = "alloc")]

use switchyard_core::{
    ActionId, Host, Op, OwnedProgram, ProgramCatalog, ProgramId, Runtime, SignalId, TaskId,
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
fn owned_program_authors_a_runnable_program() {
    let mut program = OwnedProgram::new(ProgramId(11));
    program.wait_signal(SignalId(4)).action(ActionId(1)).action(ActionId(2)).succeed();

    assert_eq!(
        program.ops(),
        &[
            Op::WaitSignal(SignalId(4)),
            Op::Action(ActionId(1)),
            Op::Action(ActionId(2)),
            Op::Succeed,
        ]
    );

    let programs = [program.as_program()];
    let mut runtime: Runtime<4, 2> = Runtime::new(ProgramCatalog::new(&programs));
    let mut host = TestHost::default();
    let task = runtime.spawn(ProgramId(11)).expect("spawn owned program");

    runtime.tick(&mut host).expect("wait tick");
    runtime.emit_signal(SignalId(4)).expect("emit signal");
    runtime.tick(&mut host).expect("wake tick");

    assert_eq!(host.actions, vec![(task.0, 1), (task.0, 2)]);
}

#[test]
fn owned_program_clear_reuses_builder_storage() {
    let mut program = OwnedProgram::new(ProgramId(12));
    program.action(ActionId(1)).succeed();
    assert_eq!(program.ops(), &[Op::Action(ActionId(1)), Op::Succeed]);

    program.clear();
    program.wait_signal(SignalId(9)).fail();

    assert_eq!(program.ops(), &[Op::WaitSignal(SignalId(9)), Op::Fail]);
}
