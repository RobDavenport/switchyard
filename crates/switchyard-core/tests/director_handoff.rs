use switchyard_core::{
    ActionId, Host, MindId, Op, Outcome, Program, ProgramCatalog, ProgramId, Runtime,
    RuntimeSnapshot, SignalId, TaskId, WaitReason,
};

const MAIN_ID: ProgramId = ProgramId(1);
const DIRECTOR_MIND: MindId = MindId(1);
const GAMEPLAY_MIND: MindId = MindId(2);
const DIRECTOR_CUE: SignalId = SignalId(1);
const GAMEPLAY_BEAT_LOCKED: SignalId = SignalId(2);
const MAIN: [Op; 8] = [
    Op::Action(ActionId(201)),
    Op::WaitSignal(DIRECTOR_CUE),
    Op::ChangeMind(GAMEPLAY_MIND),
    Op::Action(ActionId(202)),
    Op::WaitSignal(GAMEPLAY_BEAT_LOCKED),
    Op::ChangeMind(DIRECTOR_MIND),
    Op::Action(ActionId(203)),
    Op::Succeed,
];

#[derive(Default)]
struct TestHost {
    actions: Vec<(u32, u16)>,
    active_minds: Vec<MindId>,
}

impl Host for TestHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push((task.0, action.0));
    }

    fn is_mind_active(&mut self, mind: MindId) -> bool {
        self.active_minds.contains(&mind)
    }
}

#[test]
fn director_handoff_parks_until_gameplay_mind_is_active() {
    let programs = [Program::new(MAIN_ID, &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let task = runtime.spawn(MAIN_ID).expect("spawn root");
    let mut host = TestHost { active_minds: vec![DIRECTOR_MIND], ..Default::default() };

    runtime.tick(&mut host).expect("director intro");
    assert_eq!(host.actions, vec![(task.0, 201)]);

    runtime.emit_signal(DIRECTOR_CUE).expect("queue director cue");
    runtime.tick(&mut host).expect("handoff to gameplay mind");
    let parked = runtime.task(task).expect("task still present");
    assert_eq!(parked.mind_id, GAMEPLAY_MIND);
    assert_eq!(parked.wait, WaitReason::Ready);
    assert_eq!(host.actions, vec![(task.0, 201)]);

    host.active_minds = vec![GAMEPLAY_MIND];
    runtime.tick(&mut host).expect("gameplay takeover");
    assert_eq!(host.actions, vec![(task.0, 201), (task.0, 202)]);
    assert_eq!(
        runtime.task(task).expect("task waiting").wait,
        WaitReason::Signal(GAMEPLAY_BEAT_LOCKED)
    );

    runtime.emit_signal(GAMEPLAY_BEAT_LOCKED).expect("queue gameplay beat lock");
    runtime.tick(&mut host).expect("handoff back to director mind parks");
    let parked_back = runtime.task(task).expect("task still present");
    assert_eq!(parked_back.mind_id, DIRECTOR_MIND);
    assert_eq!(host.actions, vec![(task.0, 201), (task.0, 202)]);

    host.active_minds = vec![DIRECTOR_MIND];
    runtime.tick(&mut host).expect("director cleanup");
    assert_eq!(host.actions, vec![(task.0, 201), (task.0, 202), (task.0, 203)]);
    assert_eq!(runtime.task(task).expect("task finished").outcome, Outcome::Succeeded);
}

#[test]
fn director_handoff_snapshot_restore_preserves_parked_state() {
    let programs = [Program::new(MAIN_ID, &MAIN)];
    let catalog = ProgramCatalog::new(&programs);
    let mut runtime: Runtime<8, 4> = Runtime::new(catalog);
    let task = runtime.spawn(MAIN_ID).expect("spawn root");
    let mut host = TestHost { active_minds: vec![DIRECTOR_MIND], ..Default::default() };

    runtime.tick(&mut host).expect("director intro");
    runtime.emit_signal(DIRECTOR_CUE).expect("queue director cue");
    runtime.tick(&mut host).expect("handoff to gameplay mind parks");
    let snapshot: RuntimeSnapshot<8, 4> = runtime.snapshot();

    let mut restored: Runtime<8, 4> = Runtime::from_snapshot(catalog, snapshot);
    let mut restored_host = TestHost { active_minds: vec![DIRECTOR_MIND], ..Default::default() };

    restored.tick(&mut restored_host).expect("still parked while gameplay mind inactive");
    assert_eq!(restored_host.actions, Vec::<(u32, u16)>::new());
    assert_eq!(restored.task(task).expect("task still parked").mind_id, GAMEPLAY_MIND);

    restored_host.active_minds.push(GAMEPLAY_MIND);
    restored.tick(&mut restored_host).expect("restored gameplay takeover");
    assert_eq!(restored_host.actions, vec![(task.0, 202)]);
    assert_eq!(
        restored.task(task).expect("task waiting after gameplay action").wait,
        WaitReason::Signal(GAMEPLAY_BEAT_LOCKED)
    );
}
