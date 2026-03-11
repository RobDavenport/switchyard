use switchyard_core::{
    ActionId, Host, Op, Program, ProgramCatalog, ProgramId, Runtime, SignalId, TaskId, TraceEvent,
    TraceSink,
};
use switchyard_debug::TraceLog;

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
fn trace_log_records_events_from_runtime() {
    const MAIN: [Op; 3] = [Op::WaitSignal(SignalId(7)), Op::Action(ActionId(5)), Op::Succeed];

    let programs = [Program::new(ProgramId(1), &MAIN)];
    let mut runtime: Runtime<4, 2> = Runtime::new(ProgramCatalog::new(&programs));
    let mut host = TestHost::default();
    let mut trace = TraceLog::default();

    runtime.spawn_traced(ProgramId(1), &mut trace).expect("spawn root");
    runtime.tick_traced(&mut host, &mut trace).expect("wait tick");
    runtime.emit_signal_traced(SignalId(7), &mut trace).expect("emit signal");
    runtime.tick_traced(&mut host, &mut trace).expect("wake tick");

    assert_eq!(trace.events().len(), 10);
    assert!(trace.render().contains("signal_queued signal=7"));
    assert!(trace.render().contains("action_emitted task=1 action=5"));
}

#[test]
fn trace_log_clear_drops_prior_events() {
    let mut trace = TraceLog::default();
    trace.on_event(TraceEvent::SignalQueued { signal: SignalId(9) });
    assert_eq!(trace.events().len(), 1);

    trace.clear();

    assert!(trace.events().is_empty());
    assert!(trace.render().is_empty());
}
