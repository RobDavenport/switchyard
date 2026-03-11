use switchyard_core::{
    ActionId, Host, Op, Outcome, Program, ProgramCatalog, ProgramId, Runtime, SignalId, StepReport,
    TaskId, TraceEvent, TraceSink, WaitReason,
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
