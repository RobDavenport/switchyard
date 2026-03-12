use serde::{Deserialize, Serialize};
use switchyard_core::{
    ActionId, Host, Op, Outcome, PredicateId, Program, ProgramCatalog, ProgramId, Runtime,
    RuntimeSnapshot, SignalId, TaskId, TaskRecord, WaitReason,
};
use switchyard_debug::TraceLog;

const MAX_TASKS: usize = 16;
const MAX_PENDING_SIGNALS: usize = 8;
const MAIN_ID: ProgramId = ProgramId(1);
const GATE_ID: ProgramId = ProgramId(2);
const SCOUTS_ID: ProgramId = ProgramId(3);
const BOSS_INTRO_ID: ProgramId = ProgramId(4);
const ESCAPE_ID: ProgramId = ProgramId(5);
const PLAYER_COMMITTED: SignalId = SignalId(1);
const SCOUTS_READY: SignalId = SignalId(2);
const BOSS_SPOTTED: SignalId = SignalId(3);
const COLLAPSE_TRIGGERED: SignalId = SignalId(4);
const BOSS_VULNERABLE: PredicateId = PredicateId(1);

const MAIN: [Op; 9] = [
    Op::Action(ActionId(1)),
    Op::WaitSignal(PLAYER_COMMITTED),
    Op::Spawn(GATE_ID),
    Op::Spawn(SCOUTS_ID),
    Op::JoinChildren,
    Op::WaitPredicate(BOSS_VULNERABLE),
    Op::Race2(BOSS_INTRO_ID, ESCAPE_ID),
    Op::Action(ActionId(6)),
    Op::Succeed,
];
const GATE: [Op; 3] = [Op::WaitTicks(2), Op::Action(ActionId(2)), Op::Succeed];
const SCOUTS: [Op; 3] = [Op::WaitSignal(SCOUTS_READY), Op::Action(ActionId(3)), Op::Succeed];
const BOSS_INTRO: [Op; 3] = [Op::WaitSignal(BOSS_SPOTTED), Op::Action(ActionId(4)), Op::Succeed];
const ESCAPE: [Op; 3] = [Op::WaitSignal(COLLAPSE_TRIGGERED), Op::Action(ActionId(5)), Op::Succeed];

static PROGRAMS: [Program<'static>; 5] = [
    Program::new(MAIN_ID, &MAIN),
    Program::new(GATE_ID, &GATE),
    Program::new(SCOUTS_ID, &SCOUTS),
    Program::new(BOSS_INTRO_ID, &BOSS_INTRO),
    Program::new(ESCAPE_ID, &ESCAPE),
];

fn catalog() -> ProgramCatalog<'static> {
    ProgramCatalog::new(&PROGRAMS)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowcaseAction {
    pub task_id: u32,
    pub action_id: u16,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowcaseTask {
    pub task_id: u32,
    pub program_id: u16,
    pub program_label: String,
    pub parent: Option<u32>,
    pub ip: usize,
    pub outcome: String,
    pub wait: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowcaseView {
    pub title: String,
    pub subtitle: String,
    pub clock: u64,
    pub beat: String,
    pub boss_vulnerable: bool,
    pub tasks: Vec<ShowcaseTask>,
    pub actions: Vec<ShowcaseAction>,
    pub trace: String,
    pub snapshot_hint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotEnvelope {
    runtime: RuntimeSnapshotDef,
    boss_vulnerable: bool,
    actions: Vec<SnapshotActionDef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RuntimeSnapshotDef {
    clock: u64,
    next_task_id: u32,
    tasks: Vec<TaskSlotDef>,
    pending_signals: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TaskSlotDef {
    slot: usize,
    task: TaskRecordDef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TaskRecordDef {
    id: u32,
    program_id: u16,
    ip: usize,
    parent: Option<u32>,
    scope_root: u32,
    outcome: OutcomeDef,
    wait: WaitReasonDef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum OutcomeDef {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WaitReasonDef {
    Ready,
    Ticks { until_tick: u64 },
    Signal { signal: u16 },
    Predicate { predicate: u16 },
    ChildrenAll,
    Race { left: u32, right: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotActionDef {
    task_id: u32,
    action_id: u16,
}

#[derive(Default)]
struct ShowcaseHost {
    actions: Vec<ShowcaseAction>,
    boss_vulnerable: bool,
}

impl Host for ShowcaseHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push(ShowcaseAction {
            task_id: task.0,
            action_id: action.0,
            label: action_label(action).to_owned(),
        });
    }

    fn query_ready(&mut self, predicate: PredicateId) -> bool {
        predicate == BOSS_VULNERABLE && self.boss_vulnerable
    }
}

pub struct ShowcaseState {
    runtime: Runtime<'static, MAX_TASKS, MAX_PENDING_SIGNALS>,
    host: ShowcaseHost,
    trace: TraceLog,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

impl ShowcaseState {
    pub fn new() -> Self {
        let mut state = Self {
            runtime: Runtime::new(catalog()),
            host: ShowcaseHost::default(),
            trace: TraceLog::default(),
        };
        state.spawn_root().expect("spawn showcase root");
        state
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.runtime = Runtime::new(catalog());
        self.host = ShowcaseHost::default();
        self.trace.clear();
        self.spawn_root()
    }

    pub fn tick(&mut self) -> Result<(), String> {
        self.runtime
            .tick_traced(&mut self.host, &mut self.trace)
            .map(|_| ())
            .map_err(|error| format!("tick failed: {error:?}"))
    }

    pub fn emit_signal(&mut self, signal: SignalId) -> Result<(), String> {
        self.runtime
            .emit_signal_traced(signal, &mut self.trace)
            .map_err(|error| format!("emit signal failed: {error:?}"))
    }

    pub fn set_boss_vulnerable(&mut self, ready: bool) {
        self.host.boss_vulnerable = ready;
    }

    pub fn save_snapshot(&self) -> Result<String, String> {
        let envelope = SnapshotEnvelope {
            runtime: RuntimeSnapshotDef::from_runtime_snapshot(self.runtime.snapshot()),
            boss_vulnerable: self.host.boss_vulnerable,
            actions: self
                .host
                .actions
                .iter()
                .map(|action| SnapshotActionDef {
                    task_id: action.task_id,
                    action_id: action.action_id,
                })
                .collect(),
        };
        serde_json::to_string_pretty(&envelope)
            .map_err(|error| format!("snapshot encode failed: {error}"))
    }

    pub fn load_snapshot(&mut self, text: &str) -> Result<(), String> {
        let envelope: SnapshotEnvelope = serde_json::from_str(text)
            .map_err(|error| format!("snapshot decode failed: {error}"))?;
        self.runtime = Runtime::from_snapshot(catalog(), envelope.runtime.into_runtime_snapshot()?);
        self.host = ShowcaseHost {
            actions: envelope
                .actions
                .into_iter()
                .map(|action| ShowcaseAction {
                    task_id: action.task_id,
                    action_id: action.action_id,
                    label: action_label(ActionId(action.action_id)).to_owned(),
                })
                .collect(),
            boss_vulnerable: envelope.boss_vulnerable,
        };
        self.trace.clear();
        Ok(())
    }

    pub fn view(&self) -> ShowcaseView {
        ShowcaseView {
            title: "The Death of Tick".to_owned(),
            subtitle: "A switchyard showcase for structured encounter orchestration.".to_owned(),
            clock: self.runtime.clock(),
            beat: self.beat_line(),
            boss_vulnerable: self.host.boss_vulnerable,
            tasks: self.collect_tasks(),
            actions: self.host.actions.clone(),
            trace: self.trace.render(),
            snapshot_hint: "Save a snapshot before resolving the race, then restore and choose the other branch.".to_owned(),
        }
    }

    pub fn view_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.view()).map_err(|error| format!("view encode failed: {error}"))
    }

    fn spawn_root(&mut self) -> Result<(), String> {
        self.runtime
            .spawn_traced(MAIN_ID, &mut self.trace)
            .map(|_| ())
            .map_err(|error| format!("spawn failed: {error:?}"))
    }

    fn collect_tasks(&self) -> Vec<ShowcaseTask> {
        let mut tasks: Vec<ShowcaseTask> =
            self.runtime.tasks().into_iter().flatten().map(showcase_task).collect();
        tasks.sort_by_key(|task| task.task_id);
        tasks
    }

    fn beat_line(&self) -> String {
        if let Some(action) = self.host.actions.last() {
            return action.label.clone();
        }

        if let Some(root) = self.runtime.task(TaskId(1)) {
            return match root.wait {
                WaitReason::Ready => "Root is ready to advance.".to_owned(),
                WaitReason::Signal(signal) => format!("Awaiting signal: {}", signal_label(signal)),
                WaitReason::Predicate(predicate) => {
                    format!("Awaiting predicate: {}", predicate_label(predicate))
                }
                WaitReason::ChildrenAll => "Concurrent routines are still resolving.".to_owned(),
                WaitReason::Race { .. } => {
                    "Two branches are racing. Pick the fate of the scene.".to_owned()
                }
                WaitReason::Ticks { until_tick } => {
                    format!("Advancing toward clock {}.", until_tick)
                }
            };
        }

        "Showcase ready.".to_owned()
    }
}

fn showcase_task(task: TaskRecord) -> ShowcaseTask {
    ShowcaseTask {
        task_id: task.id.0,
        program_id: task.program_id.0,
        program_label: program_label(task.program_id).to_owned(),
        parent: task.parent.map(|parent| parent.0),
        ip: task.ip,
        outcome: outcome_label(task.outcome).to_owned(),
        wait: wait_reason_label(task.wait),
    }
}

fn wait_reason_label(reason: WaitReason) -> String {
    match reason {
        WaitReason::Ready => "ready".to_owned(),
        WaitReason::Ticks { until_tick } => format!("ticks until {until_tick}"),
        WaitReason::Signal(signal) => format!("signal: {}", signal_label(signal)),
        WaitReason::Predicate(predicate) => format!("predicate: {}", predicate_label(predicate)),
        WaitReason::ChildrenAll => "join_children".to_owned(),
        WaitReason::Race { left, right } => {
            format!("race between task {} and task {}", left.0, right.0)
        }
    }
}

fn program_label(program_id: ProgramId) -> &'static str {
    match program_id {
        MAIN_ID => "main encounter",
        GATE_ID => "gate rise",
        SCOUTS_ID => "scout relay",
        BOSS_INTRO_ID => "boss entrance",
        ESCAPE_ID => "collapse escape",
        _ => "unknown program",
    }
}

fn signal_label(signal: SignalId) -> &'static str {
    match signal {
        PLAYER_COMMITTED => "player committed",
        SCOUTS_READY => "scouts ready",
        BOSS_SPOTTED => "boss spotted",
        COLLAPSE_TRIGGERED => "collapse triggered",
        _ => "unknown signal",
    }
}

fn predicate_label(predicate: PredicateId) -> &'static str {
    match predicate {
        BOSS_VULNERABLE => "boss vulnerable",
        _ => "unknown predicate",
    }
}

fn action_label(action: ActionId) -> &'static str {
    match action {
        ActionId(1) => "Camera crawls through the switchyard ruins.",
        ActionId(2) => "The portcullis rises on cue.",
        ActionId(3) => "A scout throws the all-clear from the gantry.",
        ActionId(4) => "The boss steps into the lantern light.",
        ActionId(5) => "The emergency escape route tears open.",
        ActionId(6) => "The encounter resolves without a monolithic tick function.",
        _ => "Unknown action.",
    }
}

fn outcome_label(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Running => "running",
        Outcome::Succeeded => "succeeded",
        Outcome::Failed => "failed",
        Outcome::Cancelled => "cancelled",
    }
}

impl RuntimeSnapshotDef {
    fn from_runtime_snapshot(snapshot: RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>) -> Self {
        let tasks = snapshot
            .tasks
            .into_iter()
            .enumerate()
            .filter_map(|(slot, task)| {
                task.map(|task| TaskSlotDef { slot, task: TaskRecordDef::from(task) })
            })
            .collect();
        let pending_signals =
            snapshot.pending_signals.into_iter().flatten().map(|signal| signal.0).collect();
        Self { clock: snapshot.clock, next_task_id: snapshot.next_task_id, tasks, pending_signals }
    }

    fn into_runtime_snapshot(
        self,
    ) -> Result<RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>, String> {
        let mut tasks: [Option<TaskRecord>; MAX_TASKS] = core::array::from_fn(|_| None);
        for slot in self.tasks {
            if slot.slot >= MAX_TASKS {
                return Err(format!(
                    "snapshot task slot {} exceeds capacity {}",
                    slot.slot, MAX_TASKS
                ));
            }
            tasks[slot.slot] = Some(slot.task.into());
        }

        let mut pending_signals: [Option<SignalId>; MAX_PENDING_SIGNALS] =
            core::array::from_fn(|_| None);
        for (index, signal) in self.pending_signals.into_iter().enumerate() {
            if index >= MAX_PENDING_SIGNALS {
                return Err(format!(
                    "snapshot signal count {} exceeds capacity {}",
                    index + 1,
                    MAX_PENDING_SIGNALS
                ));
            }
            pending_signals[index] = Some(SignalId(signal));
        }

        Ok(RuntimeSnapshot {
            clock: self.clock,
            next_task_id: self.next_task_id,
            tasks,
            pending_signals,
        })
    }
}

impl From<TaskRecord> for TaskRecordDef {
    fn from(task: TaskRecord) -> Self {
        Self {
            id: task.id.0,
            program_id: task.program_id.0,
            ip: task.ip,
            parent: task.parent.map(|parent| parent.0),
            scope_root: task.scope_root.0,
            outcome: OutcomeDef::from(task.outcome),
            wait: WaitReasonDef::from(task.wait),
        }
    }
}

impl From<TaskRecordDef> for TaskRecord {
    fn from(task: TaskRecordDef) -> Self {
        Self {
            id: TaskId(task.id),
            program_id: ProgramId(task.program_id),
            ip: task.ip,
            parent: task.parent.map(TaskId),
            scope_root: TaskId(task.scope_root),
            outcome: Outcome::from(task.outcome),
            wait: WaitReason::from(task.wait),
        }
    }
}

impl From<Outcome> for OutcomeDef {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Running => Self::Running,
            Outcome::Succeeded => Self::Succeeded,
            Outcome::Failed => Self::Failed,
            Outcome::Cancelled => Self::Cancelled,
        }
    }
}

impl From<OutcomeDef> for Outcome {
    fn from(outcome: OutcomeDef) -> Self {
        match outcome {
            OutcomeDef::Running => Self::Running,
            OutcomeDef::Succeeded => Self::Succeeded,
            OutcomeDef::Failed => Self::Failed,
            OutcomeDef::Cancelled => Self::Cancelled,
        }
    }
}

impl From<WaitReason> for WaitReasonDef {
    fn from(reason: WaitReason) -> Self {
        match reason {
            WaitReason::Ready => Self::Ready,
            WaitReason::Ticks { until_tick } => Self::Ticks { until_tick },
            WaitReason::Signal(signal) => Self::Signal { signal: signal.0 },
            WaitReason::Predicate(predicate) => Self::Predicate { predicate: predicate.0 },
            WaitReason::ChildrenAll => Self::ChildrenAll,
            WaitReason::Race { left, right } => Self::Race { left: left.0, right: right.0 },
        }
    }
}

impl From<WaitReasonDef> for WaitReason {
    fn from(reason: WaitReasonDef) -> Self {
        match reason {
            WaitReasonDef::Ready => Self::Ready,
            WaitReasonDef::Ticks { until_tick } => Self::Ticks { until_tick },
            WaitReasonDef::Signal { signal } => Self::Signal(SignalId(signal)),
            WaitReasonDef::Predicate { predicate } => Self::Predicate(PredicateId(predicate)),
            WaitReasonDef::ChildrenAll => Self::ChildrenAll,
            WaitReasonDef::Race { left, right } => {
                Self::Race { left: TaskId(left), right: TaskId(right) }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct ShowcaseApp {
        inner: ShowcaseState,
    }

    #[wasm_bindgen]
    impl ShowcaseApp {
        #[wasm_bindgen(constructor)]
        pub fn new() -> ShowcaseApp {
            ShowcaseApp { inner: ShowcaseState::new() }
        }

        pub fn reset(&mut self) -> Result<String, JsValue> {
            self.inner.reset().map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn tick(&mut self) -> Result<String, JsValue> {
            self.inner.tick().map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn emit_signal(&mut self, signal_id: u16) -> Result<String, JsValue> {
            self.inner
                .emit_signal(SignalId(signal_id))
                .map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn set_boss_vulnerable(&mut self, ready: bool) -> Result<String, JsValue> {
            self.inner.set_boss_vulnerable(ready);
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn save_snapshot(&self) -> Result<String, JsValue> {
            self.inner.save_snapshot().map_err(|error| JsValue::from_str(&error))
        }

        pub fn load_snapshot(&mut self, snapshot: &str) -> Result<String, JsValue> {
            self.inner.load_snapshot(snapshot).map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn view_json(&self) -> Result<String, String> {
            self.inner.view_json()
        }
    }
}
