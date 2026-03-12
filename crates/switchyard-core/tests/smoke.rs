use switchyard_core::{
    ActionId, Host, HostCall, HostCallId, MindId, Op, Outcome, PredicateId, Program,
    ProgramCatalog, ProgramId, Runtime, SignalId, TaskId,
};

struct TestHost {
    actions: std::vec::Vec<(u32, u16)>,
    calls: std::vec::Vec<(u32, u16, [i32; 4])>,
    active_minds: std::vec::Vec<MindId>,
    ready_predicates: std::vec::Vec<PredicateId>,
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

    fn query_ready(&mut self, predicate: PredicateId) -> bool {
        self.ready_predicates.contains(&predicate)
    }
}

#[test]
fn wait_ticks_then_action() {
    const MAIN: [Op; 4] =
        [Op::Action(ActionId(1)), Op::WaitTicks(2), Op::Action(ActionId(2)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let root = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    let step_1 = runtime.tick(&mut host).unwrap();
    assert_eq!(step_1.actions_emitted, 1);
    assert_eq!(host.actions, vec![(root.0, 1)]);

    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions, vec![(root.0, 1)]);

    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions, vec![(root.0, 1), (root.0, 2)]);
    assert_eq!(runtime.task(root).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn host_call_invokes_host_with_fixed_args() {
    const MAIN: [Op; 2] = [Op::Call(HostCall::new(HostCallId(7), [10, 20, 30, 40])), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();

    assert_eq!(host.calls, vec![(task.0, 7, [10, 20, 30, 40])]);
}

#[test]
fn wait_signal_or_ticks_wakes_on_signal_before_timeout() {
    const MAIN: [Op; 3] =
        [Op::WaitSignalOrTicks(SignalId(7), 5), Op::Action(ActionId(4)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.emit_signal(SignalId(7)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(task_id.0, 4)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn wait_signal_or_ticks_wakes_on_timeout_without_signal() {
    const MAIN: [Op; 3] =
        [Op::WaitSignalOrTicks(SignalId(7), 2), Op::Action(ActionId(4)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(task_id.0, 4)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn wait_until_tick_wakes_at_absolute_deadline() {
    const MAIN: [Op; 3] = [Op::WaitUntilTick(3), Op::Action(ActionId(4)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(task_id.0, 4)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn wait_signal_until_tick_wakes_on_signal_before_absolute_deadline() {
    const MAIN: [Op; 3] =
        [Op::WaitSignalUntilTick(SignalId(7), 4), Op::Action(ActionId(4)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.emit_signal(SignalId(7)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(task_id.0, 4)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn wait_signal_until_tick_wakes_on_absolute_deadline_without_signal() {
    const MAIN: [Op; 3] =
        [Op::WaitSignalUntilTick(SignalId(7), 3), Op::Action(ActionId(4)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(task_id.0, 4)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn timeout_until_tick_cancels_child_and_parent_continues_when_deadline_expires() {
    const PARENT: [Op; 3] =
        [Op::TimeoutUntilTick(3, ProgramId(2)), Op::Action(ActionId(9)), Op::Succeed];
    const CHILD: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(parent.0, 9)]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn race_children_until_tick_deadline_cancels_both_children_and_parent_continues() {
    const PARENT: [Op; 3] = [
        Op::RaceChildrenUntilTick(ProgramId(2), ProgramId(3), 3),
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
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(parent.0, 9)]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn timeout_ticks_cancels_child_and_parent_continues_when_deadline_expires() {
    const PARENT: [Op; 3] =
        [Op::TimeoutTicks(2, ProgramId(2)), Op::Action(ActionId(9)), Op::Succeed];
    const CHILD: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(parent.0, 9)]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn timeout_ticks_child_completion_wins_when_it_finishes_before_deadline() {
    const PARENT: [Op; 3] =
        [Op::TimeoutTicks(3, ProgramId(2)), Op::Action(ActionId(9)), Op::Succeed];
    const CHILD: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.emit_signal(SignalId(7)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 9]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Succeeded);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn timeout_ticks_zero_does_not_spawn_child() {
    const PARENT: [Op; 3] =
        [Op::TimeoutTicks(0, ProgramId(2)), Op::Action(ActionId(9)), Op::Succeed];
    const CHILD: [Op; 2] = [Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(parent.0, 9)]);
    assert_eq!(runtime.task(TaskId(2)), None);
}

#[test]
fn race_children_or_ticks_winner_cancels_loser_before_deadline() {
    const PARENT: [Op; 3] = [
        Op::RaceChildrenOrTicks(ProgramId(2), ProgramId(3), 3),
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const LEFT: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];
    const RIGHT: [Op; 3] = [Op::WaitSignal(SignalId(8)), Op::Action(ActionId(2)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &LEFT),
        Program::new(ProgramId(3), &RIGHT),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.emit_signal(SignalId(7)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 9]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Succeeded);
    assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn race_children_or_ticks_deadline_cancels_both_children_and_parent_continues() {
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
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(parent.0, 9)]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn race_children_or_ticks_zero_does_not_spawn_children() {
    const PARENT: [Op; 3] = [
        Op::RaceChildrenOrTicks(ProgramId(2), ProgramId(3), 0),
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const LEFT: [Op; 2] = [Op::Action(ActionId(1)), Op::Succeed];
    const RIGHT: [Op; 2] = [Op::Action(ActionId(2)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &LEFT),
        Program::new(ProgramId(3), &RIGHT),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let parent = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(parent.0, 9)]);
    assert_eq!(runtime.task(TaskId(2)), None);
    assert_eq!(runtime.task(TaskId(3)), None);
}

#[test]
fn change_mind_blocks_task_until_target_mind_becomes_active() {
    const MAIN: [Op; 3] = [Op::ChangeMind(MindId(2)), Op::Action(ActionId(8)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost { active_minds: vec![MindId(1)], ..Default::default() };

    runtime.tick(&mut host).unwrap();
    assert!(host.actions.is_empty());
    assert_eq!(runtime.task(task_id).unwrap().mind_id, MindId(2));

    host.active_minds.push(MindId(2));
    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions, vec![(task_id.0, 8)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn spawn_after_change_inherits_new_mind() {
    const PARENT: [Op; 4] =
        [Op::ChangeMind(MindId(2)), Op::Spawn(ProgramId(2)), Op::JoinChildren, Op::Succeed];
    const CHILD: [Op; 2] = [Op::WaitSignal(SignalId(7)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost { active_minds: vec![MindId(1)], ..Default::default() };

    runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();

    host.active_minds.push(MindId(2));
    runtime.tick(&mut host).unwrap();

    assert_eq!(runtime.task(TaskId(2)).unwrap().mind_id, MindId(2));
    assert_eq!(runtime.task(TaskId(1)).unwrap().mind_id, MindId(2));
}

#[test]
fn predicate_wait_blocks_until_host_ready() {
    const MAIN: [Op; 3] = [Op::WaitPredicate(PredicateId(5)), Op::Action(ActionId(8)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let mut host = TestHost::default();

    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();
    assert!(host.actions.is_empty());
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Running);

    host.ready_predicates.push(PredicateId(5));
    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions, vec![(task_id.0, 8)]);
    assert_eq!(runtime.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn repeat_until_predicate_repeats_child_until_host_ready() {
    const PARENT: [Op; 3] = [
        Op::RepeatUntilPredicate(PredicateId(5), ProgramId(2)),
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const CHILD: [Op; 2] = [Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    let parent = runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1]);

    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 1]);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Running);

    host.ready_predicates.push(PredicateId(5));
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 1, 9]);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn spawn_then_join_is_ordered() {
    const PARENT: [Op; 5] = [
        Op::Spawn(ProgramId(2)),
        Op::Spawn(ProgramId(2)),
        Op::JoinChildren,
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const CHILD: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();
    assert!(host.actions.is_empty());

    runtime.emit_signal(SignalId(7)).unwrap();
    runtime.tick(&mut host).unwrap();
    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 1, 9]);
}

#[test]
fn spawn_then_join_any_advances_on_first_finished_child() {
    const PARENT: [Op; 5] = [
        Op::Spawn(ProgramId(2)),
        Op::Spawn(ProgramId(3)),
        Op::JoinAnyChildren,
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const LEFT: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];
    const RIGHT: [Op; 3] = [Op::WaitSignal(SignalId(8)), Op::Action(ActionId(2)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &LEFT),
        Program::new(ProgramId(3), &RIGHT),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    let parent = runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();
    assert_eq!(runtime.task(parent).unwrap().wait, switchyard_core::WaitReason::ChildrenAny);

    runtime.emit_signal(SignalId(8)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![2, 9]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Cancelled);
    assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, Outcome::Succeeded);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn race_cancels_loser() {
    const PARENT: [Op; 3] =
        [Op::Race2(ProgramId(2), ProgramId(3)), Op::Action(ActionId(9)), Op::Succeed];
    const FAST: [Op; 3] = [Op::WaitSignal(SignalId(1)), Op::Action(ActionId(2)), Op::Succeed];
    const SLOW: [Op; 3] = [Op::WaitSignal(SignalId(2)), Op::Action(ActionId(3)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &FAST),
        Program::new(ProgramId(3), &SLOW),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    let parent = runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();

    runtime.emit_signal(SignalId(1)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![2, 9]);
    assert_eq!(runtime.task(parent).unwrap().outcome, Outcome::Succeeded);
    assert_eq!(runtime.task(TaskId(3)).unwrap().outcome, Outcome::Cancelled);
}

#[test]
fn snapshot_restore_resumes_waiting_task() {
    const MAIN: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let task_id = runtime.spawn(ProgramId(1)).unwrap();
    let mut host = TestHost::default();

    runtime.tick(&mut host).unwrap();
    let snapshot = runtime.snapshot();

    let mut restored: Runtime<8, 4> = Runtime::from_snapshot(catalog, snapshot);
    restored.emit_signal(SignalId(7)).unwrap();
    restored.tick(&mut host).unwrap();

    assert_eq!(host.actions, vec![(task_id.0, 1)]);
    assert_eq!(restored.task(task_id).unwrap().outcome, Outcome::Succeeded);
}

#[test]
fn branch_predicate_true_runs_true_child_then_continues() {
    const PARENT: [Op; 3] = [
        Op::BranchPredicate(PredicateId(5), ProgramId(2), ProgramId(3)),
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const TRUE_BRANCH: [Op; 2] = [Op::Action(ActionId(1)), Op::Succeed];
    const FALSE_BRANCH: [Op; 2] = [Op::Action(ActionId(2)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &TRUE_BRANCH),
        Program::new(ProgramId(3), &FALSE_BRANCH),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost { ready_predicates: vec![PredicateId(5)], ..Default::default() };

    runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![1, 9]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Succeeded);
    assert_eq!(runtime.task(TaskId(3)), None);
}

#[test]
fn branch_predicate_false_runs_false_child_then_continues() {
    const PARENT: [Op; 3] = [
        Op::BranchPredicate(PredicateId(5), ProgramId(2), ProgramId(3)),
        Op::Action(ActionId(9)),
        Op::Succeed,
    ];
    const TRUE_BRANCH: [Op; 2] = [Op::Action(ActionId(1)), Op::Succeed];
    const FALSE_BRANCH: [Op; 2] = [Op::Action(ActionId(2)), Op::Succeed];

    let programs = [
        Program::new(ProgramId(1), &PARENT),
        Program::new(ProgramId(2), &TRUE_BRANCH),
        Program::new(ProgramId(3), &FALSE_BRANCH),
    ];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();

    runtime.spawn(ProgramId(1)).unwrap();
    runtime.tick(&mut host).unwrap();

    assert_eq!(host.actions.iter().map(|(_, action)| *action).collect::<Vec<_>>(), vec![2, 9]);
    assert_eq!(runtime.task(TaskId(2)).unwrap().program_id, ProgramId(3));
    assert_eq!(runtime.task(TaskId(2)).unwrap().outcome, Outcome::Succeeded);
    assert_eq!(runtime.task(TaskId(3)), None);
}
