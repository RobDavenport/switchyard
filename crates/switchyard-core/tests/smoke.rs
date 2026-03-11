use switchyard_core::{
    ActionId, Host, Op, Outcome, PredicateId, Program, ProgramCatalog, ProgramId, Runtime,
    SignalId, TaskId,
};

#[derive(Default)]
struct TestHost {
    actions: std::vec::Vec<(u32, u16)>,
    ready_predicates: std::vec::Vec<PredicateId>,
}

impl Host for TestHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push((task.0, action.0));
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
