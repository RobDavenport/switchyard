use switchyard_core::{
    ActionId, BuildError, Host, HostCall, HostCallId, MindId, Op, PredicateId, ProgramBuilder,
    ProgramCatalog, ProgramId, Runtime, TaskId,
};

#[derive(Default)]
struct TestHost {
    actions: std::vec::Vec<(u32, u16)>,
    calls: std::vec::Vec<(u32, u16, [i32; 4])>,
}

impl Host for TestHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push((task.0, action.0));
    }

    fn on_call(&mut self, task: TaskId, call: HostCall) {
        self.calls.push((task.0, call.id.0, call.args));
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

#[test]
fn builder_authors_host_call_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(12));
    builder.call(HostCallId(7), [1, 2, 3, 4]).expect("call fits");

    assert_eq!(builder.program().ops, &[Op::Call(HostCall::new(HostCallId(7), [1, 2, 3, 4]))]);
}

#[test]
fn builder_authors_wait_signal_or_ticks_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(20));
    builder.wait_signal_or_ticks(switchyard_core::SignalId(7), 3).expect("wait fits");

    assert_eq!(builder.program().ops, &[Op::WaitSignalOrTicks(switchyard_core::SignalId(7), 3)]);
}

#[test]
fn builder_authors_wait_until_tick_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(24));
    builder.wait_until_tick(7).expect("absolute wait fits");

    assert_eq!(builder.program().ops, &[Op::WaitUntilTick(7)]);
}

#[test]
fn builder_authors_wait_signal_until_tick_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(25));
    builder
        .wait_signal_until_tick(switchyard_core::SignalId(7), 9)
        .expect("absolute signal wait fits");

    assert_eq!(builder.program().ops, &[Op::WaitSignalUntilTick(switchyard_core::SignalId(7), 9)]);
}

#[test]
fn builder_authors_timeout_until_tick_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(26));
    builder.timeout_until_tick(11, ProgramId(7)).expect("absolute timeout fits");

    assert_eq!(builder.program().ops, &[Op::TimeoutUntilTick(11, ProgramId(7))]);
}

#[test]
fn builder_authors_race_children_until_tick_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(27));
    builder.race_children_until_tick(ProgramId(2), ProgramId(3), 12).expect("absolute race fits");

    assert_eq!(builder.program().ops, &[Op::RaceChildrenUntilTick(ProgramId(2), ProgramId(3), 12)]);
}

#[test]
fn builder_authors_timeout_ticks_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(21));
    builder.timeout_ticks(2, ProgramId(7)).expect("timeout fits");

    assert_eq!(builder.program().ops, &[Op::TimeoutTicks(2, ProgramId(7))]);
}

#[test]
fn builder_authors_change_mind_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(11));
    builder.change_mind(MindId(2)).expect("change fits");

    assert_eq!(builder.program().ops, &[Op::ChangeMind(MindId(2))]);
}

#[test]
fn builder_authors_branch_predicate_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(13));
    builder.branch_predicate(PredicateId(5), ProgramId(2), ProgramId(3)).expect("branch fits");

    assert_eq!(
        builder.program().ops,
        &[Op::BranchPredicate(PredicateId(5), ProgramId(2), ProgramId(3))]
    );
}

#[test]
fn builder_authors_join_any_children_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(14));
    builder.join_any_children().expect("join-any fits");

    assert_eq!(builder.program().ops, &[Op::JoinAnyChildren]);
}

#[test]
fn builder_authors_repeat_until_predicate_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(16));
    builder.repeat_until_predicate(PredicateId(5), ProgramId(2)).expect("repeat-until fits");

    assert_eq!(builder.program().ops, &[Op::RepeatUntilPredicate(PredicateId(5), ProgramId(2))]);
}

#[test]
fn builder_authors_race_children_or_ticks_op() {
    let mut builder = ProgramBuilder::<1>::new(ProgramId(19));
    builder.race_children_or_ticks(ProgramId(2), ProgramId(3), 4).expect("race-timeout fits");

    assert_eq!(builder.program().ops, &[Op::RaceChildrenOrTicks(ProgramId(2), ProgramId(3), 4)]);
}

#[test]
fn builder_sync_children_expands_to_spawn_spawn_join() {
    let mut builder = ProgramBuilder::<4>::new(ProgramId(17));
    builder.sync_children(ProgramId(2), ProgramId(3)).expect("sync fits");
    builder.succeed().expect("succeed fits");

    assert_eq!(
        builder.program().ops,
        &[Op::Spawn(ProgramId(2)), Op::Spawn(ProgramId(3)), Op::JoinChildren, Op::Succeed,]
    );
}

#[test]
fn builder_race_children_authors_race2_op() {
    let mut builder = ProgramBuilder::<2>::new(ProgramId(18));
    builder.race_children(ProgramId(2), ProgramId(3)).expect("race fits");
    builder.succeed().expect("succeed fits");

    assert_eq!(builder.program().ops, &[Op::Race2(ProgramId(2), ProgramId(3)), Op::Succeed]);
}

#[test]
fn builder_repeat_count_expands_to_spawn_join_pairs() {
    let mut builder = ProgramBuilder::<5>::new(ProgramId(15));
    builder.repeat_count(2, ProgramId(2)).expect("repeat fits");
    builder.succeed().expect("succeed fits");

    assert_eq!(
        builder.program().ops,
        &[
            Op::Spawn(ProgramId(2)),
            Op::JoinChildren,
            Op::Spawn(ProgramId(2)),
            Op::JoinChildren,
            Op::Succeed,
        ]
    );
}
