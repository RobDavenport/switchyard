use switchyard_core::{
    ActionId, BuildError, Host, Op, ProgramBuilder, ProgramCatalog, ProgramId, Runtime, TaskId,
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
fn builder_authors_a_runnable_program_in_order() {
    let mut builder = ProgramBuilder::<4>::new(ProgramId(7));
    builder
        .action(ActionId(10))
        .expect("push first action")
        .wait_ticks(1)
        .expect("push wait")
        .action(ActionId(20))
        .expect("push second action")
        .succeed()
        .expect("push succeed");

    let program = builder.program();
    assert_eq!(
        program.ops,
        &[Op::Action(ActionId(10)), Op::WaitTicks(1), Op::Action(ActionId(20)), Op::Succeed,]
    );

    let programs = [program];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let mut host = TestHost::default();
    let task = runtime.spawn(ProgramId(7)).expect("spawn program");

    runtime.tick(&mut host).expect("first tick");
    runtime.tick(&mut host).expect("second tick");

    assert_eq!(host.actions, vec![(task.0, 10), (task.0, 20)]);
}

#[test]
fn builder_rejects_ops_past_fixed_capacity() {
    let mut builder = ProgramBuilder::<2>::new(ProgramId(9));

    builder.action(ActionId(1)).expect("first action fits");
    builder.succeed().expect("second op fits");

    let error = builder.fail().expect_err("third op should exceed fixed capacity");

    assert_eq!(error, BuildError::CapacityExceeded);
    assert_eq!(builder.program().ops, &[Op::Action(ActionId(1)), Op::Succeed]);
}
