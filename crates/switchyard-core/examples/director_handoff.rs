use switchyard_core::{
    ActionId, Host, MindId, ProgramBuilder, ProgramCatalog, ProgramId, Runtime, RuntimeSnapshot,
    SignalId, TaskId,
};

const MAX_TASKS: usize = 8;
const MAX_PENDING_SIGNALS: usize = 4;
const MAIN_ID: ProgramId = ProgramId(1);
const DIRECTOR_MIND: MindId = MindId(1);
const GAMEPLAY_MIND: MindId = MindId(2);
const DIRECTOR_CUE: SignalId = SignalId(1);
const GAMEPLAY_BEAT_LOCKED: SignalId = SignalId(2);

struct ConsoleHost {
    active_mind: MindId,
}

impl ConsoleHost {
    fn set_active_mind(&mut self, mind: MindId) {
        self.active_mind = mind;
        println!("host switches active mind -> {}", mind.0);
    }
}

impl Host for ConsoleHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        println!("task={} action={} {}", task.0, action.0, action_label(action));
    }

    fn is_mind_active(&mut self, mind: MindId) -> bool {
        self.active_mind == mind
    }
}

fn main() {
    let mut main = ProgramBuilder::<8>::new(MAIN_ID);
    main.action(ActionId(201))
        .expect("director intro action")
        .wait_signal(DIRECTOR_CUE)
        .expect("wait for director cue")
        .change_mind(GAMEPLAY_MIND)
        .expect("handoff to gameplay")
        .action(ActionId(202))
        .expect("gameplay takeover action")
        .wait_signal(GAMEPLAY_BEAT_LOCKED)
        .expect("wait for gameplay beat lock")
        .change_mind(DIRECTOR_MIND)
        .expect("handoff back to director")
        .action(ActionId(203))
        .expect("director cleanup action")
        .succeed()
        .expect("main succeed");

    let programs = [main.program()];
    let catalog = ProgramCatalog::new(&programs);

    let mut runtime: Runtime<MAX_TASKS, MAX_PENDING_SIGNALS> = Runtime::new(catalog);
    let mut host = ConsoleHost { active_mind: DIRECTOR_MIND };
    runtime.spawn(MAIN_ID).expect("spawn root");

    runtime.tick(&mut host).expect("director intro");
    runtime.emit_signal(DIRECTOR_CUE).expect("queue director cue");
    runtime.tick(&mut host).expect("handoff parks on gameplay mind");
    println!("snapshot saved while gameplay mind is parked at clock {}", runtime.clock());
    let parked_snapshot: RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS> = runtime.snapshot();

    host.set_active_mind(GAMEPLAY_MIND);
    runtime.tick(&mut host).expect("gameplay takes over");
    runtime.emit_signal(GAMEPLAY_BEAT_LOCKED).expect("queue gameplay beat locked");
    runtime.tick(&mut host).expect("handoff back to director parks");
    host.set_active_mind(DIRECTOR_MIND);
    runtime.tick(&mut host).expect("director cleanup");

    println!("restoring parked snapshot and replaying the same handoff deterministically");
    let mut restored: Runtime<MAX_TASKS, MAX_PENDING_SIGNALS> =
        Runtime::from_snapshot(catalog, parked_snapshot);
    let mut restored_host = ConsoleHost { active_mind: DIRECTOR_MIND };
    restored.tick(&mut restored_host).expect("still parked on gameplay mind");
    restored_host.set_active_mind(GAMEPLAY_MIND);
    restored.tick(&mut restored_host).expect("restored gameplay takeover");
    restored.emit_signal(GAMEPLAY_BEAT_LOCKED).expect("queue restored gameplay beat lock");
    restored.tick(&mut restored_host).expect("restored handoff back to director parks");
    restored_host.set_active_mind(DIRECTOR_MIND);
    restored.tick(&mut restored_host).expect("restored director cleanup");
}

fn action_label(action: ActionId) -> &'static str {
    match action {
        ActionId(201) => "director frames the next playable beat",
        ActionId(202) => "gameplay mind takes control",
        ActionId(203) => "director regains control for cleanup",
        _ => "unknown action",
    }
}
