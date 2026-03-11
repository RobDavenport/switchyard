use switchyard_core::{
    ActionId, Host, ProgramBuilder, ProgramCatalog, ProgramId, Runtime, SignalId, TaskId,
};

struct ConsoleHost;

impl Host for ConsoleHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        println!("task={} action={}", task.0, action.0);
    }
}

fn main() {
    let mut intro = ProgramBuilder::<4>::new(ProgramId(1));
    intro
        .spawn(ProgramId(2))
        .expect("spawn child op")
        .join_children()
        .expect("join children op")
        .action(ActionId(99))
        .expect("final action op")
        .succeed()
        .expect("intro succeed op");

    let mut child = ProgramBuilder::<3>::new(ProgramId(2));
    child
        .wait_signal(SignalId(7))
        .expect("wait signal op")
        .action(ActionId(1))
        .expect("child action op")
        .succeed()
        .expect("child succeed op");

    let programs = [intro.program(), child.program()];

    let mut runtime: Runtime<8, 4> = Runtime::new(ProgramCatalog::new(&programs));
    let mut host = ConsoleHost;
    let _root = runtime.spawn(ProgramId(1)).expect("spawn root");

    runtime.tick(&mut host).expect("first tick");
    runtime.emit_signal(SignalId(7)).expect("emit signal");
    runtime.tick(&mut host).expect("second tick");
}
