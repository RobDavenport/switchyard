#![cfg(feature = "alloc")]

use switchyard_core::{
    ActionId, Host, HostCall, HostCallId, MindId, Op, OwnedProgram, PredicateId, ProgramCatalog,
    ProgramId, Runtime, SignalId, TaskId,
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

#[test]
fn owned_program_authors_host_call_op() {
    let mut program = OwnedProgram::new(ProgramId(13));
    program.call(HostCallId(7), [1, 2, 3, 4]);

    assert_eq!(program.ops(), &[Op::Call(HostCall::new(HostCallId(7), [1, 2, 3, 4]))]);
}

#[test]
fn owned_program_authors_wait_signal_or_ticks_op() {
    let mut program = OwnedProgram::new(ProgramId(21));
    program.wait_signal_or_ticks(SignalId(7), 3);

    assert_eq!(program.ops(), &[Op::WaitSignalOrTicks(SignalId(7), 3)]);
}

#[test]
fn owned_program_authors_wait_until_tick_op() {
    let mut program = OwnedProgram::new(ProgramId(24));
    program.wait_until_tick(7);

    assert_eq!(program.ops(), &[Op::WaitUntilTick(7)]);
}

#[test]
fn owned_program_authors_wait_signal_until_tick_op() {
    let mut program = OwnedProgram::new(ProgramId(25));
    program.wait_signal_until_tick(SignalId(7), 9);

    assert_eq!(program.ops(), &[Op::WaitSignalUntilTick(SignalId(7), 9)]);
}

#[test]
fn owned_program_authors_timeout_until_tick_op() {
    let mut program = OwnedProgram::new(ProgramId(26));
    program.timeout_until_tick(11, ProgramId(7));

    assert_eq!(program.ops(), &[Op::TimeoutUntilTick(11, ProgramId(7))]);
}

#[test]
fn owned_program_authors_race_children_until_tick_op() {
    let mut program = OwnedProgram::new(ProgramId(27));
    program.race_children_until_tick(ProgramId(2), ProgramId(3), 12);

    assert_eq!(program.ops(), &[Op::RaceChildrenUntilTick(ProgramId(2), ProgramId(3), 12)]);
}

#[test]
fn owned_program_authors_timeout_ticks_op() {
    let mut program = OwnedProgram::new(ProgramId(22));
    program.timeout_ticks(2, ProgramId(7));

    assert_eq!(program.ops(), &[Op::TimeoutTicks(2, ProgramId(7))]);
}

#[test]
fn owned_program_authors_change_mind_op() {
    let mut program = OwnedProgram::new(ProgramId(19));
    program.change_mind(MindId(2));

    assert_eq!(program.ops(), &[Op::ChangeMind(MindId(2))]);
}

#[test]
fn owned_program_authors_branch_predicate_op() {
    let mut program = OwnedProgram::new(ProgramId(14));
    program.branch_predicate(PredicateId(5), ProgramId(2), ProgramId(3));

    assert_eq!(program.ops(), &[Op::BranchPredicate(PredicateId(5), ProgramId(2), ProgramId(3))]);
}

#[test]
fn owned_program_authors_join_any_children_op() {
    let mut program = OwnedProgram::new(ProgramId(15));
    program.join_any_children();

    assert_eq!(program.ops(), &[Op::JoinAnyChildren]);
}

#[test]
fn owned_program_authors_repeat_until_predicate_op() {
    let mut program = OwnedProgram::new(ProgramId(20));
    program.repeat_until_predicate(PredicateId(5), ProgramId(2));

    assert_eq!(program.ops(), &[Op::RepeatUntilPredicate(PredicateId(5), ProgramId(2))]);
}

#[test]
fn owned_program_authors_race_children_or_ticks_op() {
    let mut program = OwnedProgram::new(ProgramId(23));
    program.race_children_or_ticks(ProgramId(2), ProgramId(3), 4);

    assert_eq!(program.ops(), &[Op::RaceChildrenOrTicks(ProgramId(2), ProgramId(3), 4)]);
}

#[test]
fn owned_program_sync_children_expands_to_spawn_spawn_join() {
    let mut program = OwnedProgram::new(ProgramId(17));
    program.sync_children(ProgramId(2), ProgramId(3)).succeed();

    assert_eq!(
        program.ops(),
        &[Op::Spawn(ProgramId(2)), Op::Spawn(ProgramId(3)), Op::JoinChildren, Op::Succeed,]
    );
}

#[test]
fn owned_program_race_children_authors_race2_op() {
    let mut program = OwnedProgram::new(ProgramId(18));
    program.race_children(ProgramId(2), ProgramId(3)).succeed();

    assert_eq!(program.ops(), &[Op::Race2(ProgramId(2), ProgramId(3)), Op::Succeed]);
}

#[test]
fn owned_program_repeat_count_expands_to_spawn_join_pairs() {
    let mut program = OwnedProgram::new(ProgramId(16));
    program.repeat_count(2, ProgramId(2)).succeed();

    assert_eq!(
        program.ops(),
        &[
            Op::Spawn(ProgramId(2)),
            Op::JoinChildren,
            Op::Spawn(ProgramId(2)),
            Op::JoinChildren,
            Op::Succeed,
        ]
    );
}
