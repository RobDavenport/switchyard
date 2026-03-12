use switchyard_core::{
    ActionId, Host, HostCall, HostCallId, ProgramBuilder, ProgramCatalog, ProgramId, Runtime,
    SignalId, TaskId,
};

const MAX_TASKS: usize = 16;
const MAX_PENDING_SIGNALS: usize = 8;
const MAIN_ID: ProgramId = ProgramId(1);
const WINDUP_ID: ProgramId = ProgramId(2);
const BARRAGE_ID: ProgramId = ProgramId(3);
const BURST_ID: ProgramId = ProgramId(4);
const CORE_BREAK_ID: ProgramId = ProgramId(5);
const ENRAGE_ID: ProgramId = ProgramId(6);
const WAVE_STARTED: SignalId = SignalId(1);
const CORE_BROKEN: SignalId = SignalId(3);

struct ConsoleHost;

impl Host for ConsoleHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        println!("task={} action={} {}", task.0, action.0, action_label(action));
    }

    fn on_call(&mut self, task: TaskId, call: HostCall) {
        println!("task={} call={} {}", task.0, call.id.0, call_label(call));
    }
}

fn main() {
    let mut main = ProgramBuilder::<8>::new(MAIN_ID);
    main.action(ActionId(101))
        .expect("main intro action")
        .wait_signal(WAVE_STARTED)
        .expect("wait for encounter start")
        .sync_children(WINDUP_ID, BARRAGE_ID)
        .expect("sync windup and barrage")
        .race_children_until_tick(CORE_BREAK_ID, ENRAGE_ID, 9)
        .expect("race core break against enrage")
        .action(ActionId(106))
        .expect("phase resolution action")
        .succeed()
        .expect("main succeed");

    let mut windup = ProgramBuilder::<4>::new(WINDUP_ID);
    windup
        .wait_ticks(1)
        .expect("windup delay")
        .call(HostCallId(1), [90, 128, 16, 2])
        .expect("windup host call")
        .action(ActionId(102))
        .expect("windup action")
        .succeed()
        .expect("windup succeed");

    let mut barrage = ProgramBuilder::<9>::new(BARRAGE_ID);
    barrage.repeat_count(4, BURST_ID).expect("burst repeat").succeed().expect("barrage succeed");

    let mut burst = ProgramBuilder::<3>::new(BURST_ID);
    burst
        .call(HostCallId(1), [1, 96, 24, 5])
        .expect("burst host call")
        .wait_ticks(1)
        .expect("burst cadence")
        .succeed()
        .expect("burst succeed");

    let mut core_break = ProgramBuilder::<3>::new(CORE_BREAK_ID);
    core_break
        .wait_signal_until_tick(CORE_BROKEN, 9)
        .expect("wait for core break")
        .action(ActionId(104))
        .expect("core break action")
        .succeed()
        .expect("core break succeed");

    let mut enrage = ProgramBuilder::<3>::new(ENRAGE_ID);
    enrage
        .wait_until_tick(8)
        .expect("wait for enrage deadline")
        .action(ActionId(105))
        .expect("enrage action")
        .succeed()
        .expect("enrage succeed");

    let programs = [
        main.program(),
        windup.program(),
        barrage.program(),
        burst.program(),
        core_break.program(),
        enrage.program(),
    ];

    let mut runtime: Runtime<MAX_TASKS, MAX_PENDING_SIGNALS> =
        Runtime::new(ProgramCatalog::new(&programs));
    let mut host = ConsoleHost;
    runtime.spawn(MAIN_ID).expect("spawn root");

    runtime.tick(&mut host).expect("intro tick");
    runtime.emit_signal(WAVE_STARTED).expect("start phase");
    runtime.tick(&mut host).expect("spawn windup and barrage");
    runtime.tick(&mut host).expect("advance windup and barrage");
    runtime.tick(&mut host).expect("advance barrage");
    runtime.tick(&mut host).expect("advance barrage");
    runtime.tick(&mut host).expect("enter core-break race");
    runtime.emit_signal(CORE_BROKEN).expect("break exposed core");
    runtime.tick(&mut host).expect("resolve race");
}

fn action_label(action: ActionId) -> &'static str {
    match action {
        ActionId(101) => "boss frame enters",
        ActionId(102) => "windup shutters open",
        ActionId(104) => "player breaks the core",
        ActionId(105) => "boss enrages",
        ActionId(106) => "phase resolves cleanly",
        _ => "unknown action",
    }
}

fn call_label(call: HostCall) -> String {
    match call.id {
        HostCallId(1) => format!(
            "spawn projectile pattern={} x={} y={} speed={}",
            call.args[0], call.args[1], call.args[2], call.args[3]
        ),
        _ => format!(
            "host call {} args=[{}, {}, {}, {}]",
            call.id.0, call.args[0], call.args[1], call.args[2], call.args[3]
        ),
    }
}
