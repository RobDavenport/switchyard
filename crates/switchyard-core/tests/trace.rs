use switchyard_core::{
    ActionId, Host, MindId, Op, Outcome, Program, ProgramCatalog, ProgramId, Runtime, SignalId,
    StepReport, TaskId, TraceEvent, TraceSink, WaitReason,
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

#[derive(Default)]
struct Recorder {
    events: std::vec::Vec<TraceEvent>,
}

impl TraceSink for Recorder {
    fn on_event(&mut self, event: TraceEvent) {
        self.events.push(event);
    }
}

#[test]
fn trace_reports_signal_wait_wake_action_and_finish_in_order() {
    const MAIN: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let mut host = TestHost::default();
    let mut trace = Recorder::default();

    let root = runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("first tick");
    runtime.emit_signal_traced(SignalId(7), &mut trace).expect("queue signal");
    runtime.tick_traced(&mut host, &mut trace).expect("second tick");

    assert_eq!(
        trace.events,
        vec![
            TraceEvent::TaskSpawned {
                task: root,
                program_id: ProgramId(1),
                parent: None,
                scope_root: root,
            },
            TraceEvent::TickStarted { clock: 1 },
            TraceEvent::TaskWaiting { task: root, reason: WaitReason::Signal(SignalId(7)) },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 1, actions_emitted: 0, progress_made: true },
            },
            TraceEvent::SignalQueued { signal: SignalId(7) },
            TraceEvent::TickStarted { clock: 2 },
            TraceEvent::TaskWoken { task: root, reason: WaitReason::Signal(SignalId(7)) },
            TraceEvent::ActionEmitted { task: root, action: ActionId(1) },
            TraceEvent::TaskFinished { task: root, outcome: Outcome::Succeeded },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 2, actions_emitted: 1, progress_made: true },
            },
        ]
    );
}

#[test]
fn trace_reports_signal_or_timeout_wait_and_timeout_wake_in_order() {
    const MAIN: [Op; 3] =
        [Op::WaitSignalOrTicks(SignalId(7), 2), Op::Action(ActionId(4)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let mut host = TestHost::default();
    let mut trace = Recorder::default();

    let root = runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("first tick");
    runtime.tick_traced(&mut host, &mut trace).expect("second tick");
    runtime.tick_traced(&mut host, &mut trace).expect("third tick");

    assert_eq!(
        trace.events,
        vec![
            TraceEvent::TaskSpawned {
                task: root,
                program_id: ProgramId(1),
                parent: None,
                scope_root: root,
            },
            TraceEvent::TickStarted { clock: 1 },
            TraceEvent::TaskWaiting {
                task: root,
                reason: WaitReason::SignalOrTicks { signal: SignalId(7), until_tick: 3 },
            },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 1, actions_emitted: 0, progress_made: true },
            },
            TraceEvent::TickStarted { clock: 2 },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 2, actions_emitted: 0, progress_made: false },
            },
            TraceEvent::TickStarted { clock: 3 },
            TraceEvent::TaskWoken {
                task: root,
                reason: WaitReason::SignalOrTicks { signal: SignalId(7), until_tick: 3 },
            },
            TraceEvent::ActionEmitted { task: root, action: ActionId(4) },
            TraceEvent::TaskFinished { task: root, outcome: Outcome::Succeeded },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 3, actions_emitted: 1, progress_made: true },
            },
        ]
    );
}

#[test]
fn trace_reports_timeout_child_wait_cancellation_and_parent_resume_in_order() {
    const PARENT: [Op; 3] =
        [Op::TimeoutTicks(2, ProgramId(2)), Op::Action(ActionId(9)), Op::Succeed];
    const CHILD: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &PARENT), Program::new(ProgramId(2), &CHILD)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let mut host = TestHost::default();
    let mut trace = Recorder::default();

    let root = runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("first tick");
    runtime.tick_traced(&mut host, &mut trace).expect("second tick");
    runtime.tick_traced(&mut host, &mut trace).expect("third tick");

    assert_eq!(
        trace.events,
        vec![
            TraceEvent::TaskSpawned {
                task: root,
                program_id: ProgramId(1),
                parent: None,
                scope_root: root,
            },
            TraceEvent::TickStarted { clock: 1 },
            TraceEvent::TaskSpawned {
                task: TaskId(2),
                program_id: ProgramId(2),
                parent: Some(root),
                scope_root: root,
            },
            TraceEvent::TaskWaiting {
                task: root,
                reason: WaitReason::Timeout { child: TaskId(2), until_tick: 3 },
            },
            TraceEvent::TaskWaiting { task: TaskId(2), reason: WaitReason::Signal(SignalId(7)) },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 1, actions_emitted: 0, progress_made: true },
            },
            TraceEvent::TickStarted { clock: 2 },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 2, actions_emitted: 0, progress_made: false },
            },
            TraceEvent::TickStarted { clock: 3 },
            TraceEvent::TaskFinished { task: TaskId(2), outcome: Outcome::Cancelled },
            TraceEvent::TaskWoken {
                task: root,
                reason: WaitReason::Timeout { child: TaskId(2), until_tick: 3 },
            },
            TraceEvent::ActionEmitted { task: root, action: ActionId(9) },
            TraceEvent::TaskFinished { task: root, outcome: Outcome::Succeeded },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 3, actions_emitted: 1, progress_made: true },
            },
        ]
    );
}

#[test]
fn trace_reports_race_or_timeout_resolution_in_order() {
    const PARENT: [Op; 3] = [
        Op::RaceChildrenOrTicks(ProgramId(2), ProgramId(3), 2),
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
    let mut trace = Recorder::default();

    let root = runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("first tick");
    runtime.tick_traced(&mut host, &mut trace).expect("second tick");
    runtime.tick_traced(&mut host, &mut trace).expect("third tick");

    let left = TaskId(2);
    let right = TaskId(3);

    assert_eq!(
        trace.events,
        vec![
            TraceEvent::TaskSpawned {
                task: root,
                program_id: ProgramId(1),
                parent: None,
                scope_root: root,
            },
            TraceEvent::TickStarted { clock: 1 },
            TraceEvent::TaskSpawned {
                task: left,
                program_id: ProgramId(2),
                parent: Some(root),
                scope_root: root,
            },
            TraceEvent::TaskSpawned {
                task: right,
                program_id: ProgramId(3),
                parent: Some(root),
                scope_root: root,
            },
            TraceEvent::TaskWaiting {
                task: root,
                reason: WaitReason::RaceOrTicks { left, right, until_tick: 3 },
            },
            TraceEvent::TaskWaiting { task: left, reason: WaitReason::Signal(SignalId(7)) },
            TraceEvent::TaskWaiting { task: right, reason: WaitReason::Signal(SignalId(8)) },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 1, actions_emitted: 0, progress_made: true },
            },
            TraceEvent::TickStarted { clock: 2 },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 2, actions_emitted: 0, progress_made: false },
            },
            TraceEvent::TickStarted { clock: 3 },
            TraceEvent::TaskFinished { task: left, outcome: Outcome::Cancelled },
            TraceEvent::TaskFinished { task: right, outcome: Outcome::Cancelled },
            TraceEvent::TaskWoken {
                task: root,
                reason: WaitReason::RaceOrTicks { left, right, until_tick: 3 },
            },
            TraceEvent::ActionEmitted { task: root, action: ActionId(9) },
            TraceEvent::TaskFinished { task: root, outcome: Outcome::Succeeded },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 3, actions_emitted: 1, progress_made: true },
            },
        ]
    );
}

#[test]
fn trace_reports_race_winner_and_loser_cancellation_in_order() {
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
    let mut trace = Recorder::default();

    let root = runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("first tick");
    runtime.emit_signal_traced(SignalId(1), &mut trace).expect("queue fast signal");
    runtime.tick_traced(&mut host, &mut trace).expect("second tick");

    let left = TaskId(2);
    let right = TaskId(3);

    assert_eq!(
        trace.events,
        vec![
            TraceEvent::TaskSpawned {
                task: root,
                program_id: ProgramId(1),
                parent: None,
                scope_root: root,
            },
            TraceEvent::TickStarted { clock: 1 },
            TraceEvent::TaskSpawned {
                task: left,
                program_id: ProgramId(2),
                parent: Some(root),
                scope_root: root,
            },
            TraceEvent::TaskSpawned {
                task: right,
                program_id: ProgramId(3),
                parent: Some(root),
                scope_root: root,
            },
            TraceEvent::TaskWaiting { task: root, reason: WaitReason::Race { left, right } },
            TraceEvent::TaskWaiting { task: left, reason: WaitReason::Signal(SignalId(1)) },
            TraceEvent::TaskWaiting { task: right, reason: WaitReason::Signal(SignalId(2)) },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 1, actions_emitted: 0, progress_made: true },
            },
            TraceEvent::SignalQueued { signal: SignalId(1) },
            TraceEvent::TickStarted { clock: 2 },
            TraceEvent::TaskWoken { task: left, reason: WaitReason::Signal(SignalId(1)) },
            TraceEvent::ActionEmitted { task: left, action: ActionId(2) },
            TraceEvent::TaskFinished { task: left, outcome: Outcome::Succeeded },
            TraceEvent::TaskFinished { task: right, outcome: Outcome::Cancelled },
            TraceEvent::TaskWoken { task: root, reason: WaitReason::Race { left, right } },
            TraceEvent::ActionEmitted { task: root, action: ActionId(9) },
            TraceEvent::TaskFinished { task: root, outcome: Outcome::Succeeded },
            TraceEvent::TickCompleted {
                report: StepReport { clock: 2, actions_emitted: 2, progress_made: true },
            },
        ]
    );
}

#[test]
fn trace_records_task_mind_changed() {
    const MAIN: [Op; 3] = [Op::ChangeMind(MindId(2)), Op::Action(ActionId(1)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<4, 2> = Runtime::new(catalog);
    let mut host = TestHost { active_minds: vec![MindId(1)], ..Default::default() };
    let mut trace = Recorder::default();

    let root = runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("change mind");

    assert!(trace.events.contains(&TraceEvent::TaskMindChanged {
        task: root,
        from: MindId(1),
        to: MindId(2),
    }));
}
