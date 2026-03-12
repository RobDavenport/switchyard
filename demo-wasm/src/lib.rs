use serde::{Deserialize, Serialize};
use switchyard_core::{
    ActionId, CompileError, Host, HostCall, HostCallId, MindId, OpDocument, Outcome,
    OwnedProgramCatalog, PredicateId, ProgramCatalogDocument, ProgramDocument, ProgramId, Runtime,
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
const PHASE_WINDUP_ID: ProgramId = ProgramId(21);
const PATTERN_WINDOW_ID: ProgramId = ProgramId(22);
const CORE_BREAK_ID: ProgramId = ProgramId(23);
const ENRAGE_ID: ProgramId = ProgramId(24);
const LEFT_SWEEP_ID: ProgramId = ProgramId(25);
const RIGHT_SWEEP_ID: ProgramId = ProgramId(26);
const LEFT_BURST_ID: ProgramId = ProgramId(27);
const RIGHT_BURST_ID: ProgramId = ProgramId(28);
const MULTIMIND_STAGE_ID: ProgramId = ProgramId(41);
const PLAYER_COMMITTED: SignalId = SignalId(1);
const SCOUTS_READY: SignalId = SignalId(2);
const BOSS_SPOTTED: SignalId = SignalId(3);
const COLLAPSE_TRIGGERED: SignalId = SignalId(4);
const BOSS_VULNERABLE: PredicateId = PredicateId(1);
const ENGAGE_BOSS_PHASE: SignalId = SignalId(1);
const WING_DRONES_CLEARED: SignalId = SignalId(2);
const BREAK_BOSS_CORE: SignalId = SignalId(3);
const BOMB_TRIGGERED: SignalId = SignalId(4);
const CORE_EXPOSED: PredicateId = PredicateId(1);
const DIRECTOR_CUE: SignalId = SignalId(1);
const GAMEPLAY_BEAT_LOCKED: SignalId = SignalId(2);
const DIRECTOR_MIND: MindId = MindId(1);
const GAMEPLAY_MIND: MindId = MindId(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ShowcasePreset {
    DeathOfTick,
    ShootemupBoss,
    MultiMind,
}

impl ShowcasePreset {
    const fn as_str(self) -> &'static str {
        match self {
            Self::DeathOfTick => "encounter",
            Self::ShootemupBoss => "shootemup",
            Self::MultiMind => "multimind",
        }
    }

    fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "encounter" | "death_of_tick" => Ok(Self::DeathOfTick),
            "shootemup" | "shootemup_boss" => Ok(Self::ShootemupBoss),
            "multimind" | "multi_mind" | "mind_the_gap" => Ok(Self::MultiMind),
            _ => Err(format!("unknown preset: {raw}")),
        }
    }
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
    pub mind: u16,
    pub program_label: String,
    pub parent: Option<u32>,
    pub ip: usize,
    pub outcome: String,
    pub wait: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowcaseView {
    pub preset: String,
    pub title: String,
    pub subtitle: String,
    pub clock: u64,
    pub beat: String,
    pub active_mind: u16,
    pub boss_vulnerable: bool,
    pub tasks: Vec<ShowcaseTask>,
    pub actions: Vec<ShowcaseAction>,
    pub trace: String,
    pub snapshot_hint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotEnvelope {
    preset: ShowcasePreset,
    custom_script_loaded: bool,
    script: ProgramCatalogDocument,
    runtime: RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>,
    active_mind: u16,
    boss_vulnerable: bool,
    actions: Vec<SnapshotActionDef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CliSnapshotDocument {
    clock: u64,
    next_task_id: u32,
    tasks: Vec<Option<CliTaskRecord>>,
    pending_signals: Vec<Option<SignalId>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CliTaskRecord {
    id: TaskId,
    program_id: ProgramId,
    mind_id: MindId,
    ip: usize,
    parent: Option<TaskId>,
    scope_root: TaskId,
    outcome: CliOutcome,
    wait: CliWaitReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CliOutcome {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CliWaitReason {
    Ready,
    Ticks { until_tick: u64 },
    Signal { signal: SignalId },
    Predicate { predicate: PredicateId },
    SignalOrTicks { signal: SignalId, until_tick: u64 },
    RaceOrTicks { left: TaskId, right: TaskId, until_tick: u64 },
    Timeout { child: TaskId, until_tick: u64 },
    RepeatUntilPredicate { predicate: PredicateId, resume_at_tick: u64 },
    ChildrenAll,
    ChildrenAny,
    Race { left: TaskId, right: TaskId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotActionDef {
    task_id: u32,
    action_id: u16,
    label: String,
}

#[derive(Default)]
struct ShowcaseHost {
    actions: Vec<ShowcaseAction>,
    active_mind: MindId,
    boss_vulnerable: bool,
}

impl Host for ShowcaseHost {
    fn on_action(&mut self, task: TaskId, action: ActionId) {
        self.actions.push(ShowcaseAction {
            task_id: task.0,
            action_id: action.0,
            label: action_label(action),
        });
    }

    fn on_call(&mut self, task: TaskId, call: HostCall) {
        self.actions.push(ShowcaseAction {
            task_id: task.0,
            action_id: call.id.0,
            label: call_label(call),
        });
    }

    fn is_mind_active(&mut self, mind: MindId) -> bool {
        self.active_mind == mind
    }

    fn query_ready(&mut self, predicate: PredicateId) -> bool {
        predicate == CORE_EXPOSED && self.boss_vulnerable
    }
}

pub struct ShowcaseState {
    preset: ShowcasePreset,
    custom_script_loaded: bool,
    script: ProgramCatalogDocument,
    catalog: OwnedProgramCatalog,
    snapshot: RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>,
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
        let preset = ShowcasePreset::DeathOfTick;
        let script = script_document_for_preset(preset);
        let catalog = script.compile().expect("compile default showcase script");
        let mut state = Self {
            preset,
            custom_script_loaded: false,
            script,
            catalog,
            snapshot: RuntimeSnapshot::default(),
            host: ShowcaseHost { active_mind: MindId(1), ..Default::default() },
            trace: TraceLog::default(),
        };
        state.spawn_root().expect("spawn showcase root");
        state
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.reset_runtime()
    }

    pub fn reset_script(&mut self) -> Result<(), String> {
        self.load_builtin_preset(self.preset)
    }

    pub fn script_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.script)
            .map_err(|error| format!("script encode failed: {error}"))
    }

    pub fn export_cli_catalog_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.script)
            .map_err(|error| format!("cli catalog encode failed: {error}"))
    }

    pub fn load_script(&mut self, text: &str) -> Result<(), String> {
        let script: ProgramCatalogDocument =
            serde_json::from_str(text).map_err(|error| format!("script decode failed: {error}"))?;
        let catalog = script
            .compile()
            .map_err(|error| format!("script compile failed: {}", compile_error_label(error)))?;
        if catalog.get(MAIN_ID).is_none() {
            return Err(format!("script compile failed: root program {} is missing", MAIN_ID.0));
        }

        self.custom_script_loaded = true;
        self.script = script;
        self.catalog = catalog;
        self.reset_runtime()
    }

    pub fn select_preset(&mut self, preset: &str) -> Result<(), String> {
        let preset = ShowcasePreset::parse(preset)?;
        self.load_builtin_preset(preset)
    }

    pub fn load_preset(&mut self, preset: &str) -> Result<(), String> {
        self.select_preset(preset)
    }

    pub fn tick(&mut self) -> Result<(), String> {
        self.with_runtime(|runtime, host, trace| {
            runtime
                .tick_traced(host, trace)
                .map(|_| ())
                .map_err(|error| format!("tick failed: {error:?}"))
        })
    }

    pub fn emit_signal(&mut self, signal: SignalId) -> Result<(), String> {
        self.with_runtime(|runtime, _host, trace| {
            runtime
                .emit_signal_traced(signal, trace)
                .map_err(|error| format!("emit signal failed: {error:?}"))
        })
    }

    pub fn set_boss_vulnerable(&mut self, ready: bool) {
        self.host.boss_vulnerable = ready;
    }

    pub fn set_active_mind(&mut self, mind: MindId) {
        self.host.active_mind = mind;
    }

    pub fn save_snapshot(&self) -> Result<String, String> {
        let envelope = SnapshotEnvelope {
            preset: self.preset,
            custom_script_loaded: self.custom_script_loaded,
            script: self.script.clone(),
            runtime: self.snapshot,
            active_mind: self.host.active_mind.0,
            boss_vulnerable: self.host.boss_vulnerable,
            actions: self
                .host
                .actions
                .iter()
                .map(|action| SnapshotActionDef {
                    task_id: action.task_id,
                    action_id: action.action_id,
                    label: action.label.clone(),
                })
                .collect(),
        };
        serde_json::to_string_pretty(&envelope)
            .map_err(|error| format!("snapshot encode failed: {error}"))
    }

    pub fn export_cli_runtime_snapshot_json(&self) -> Result<String, String> {
        let snapshot = CliSnapshotDocument {
            clock: self.snapshot.clock,
            next_task_id: self.snapshot.next_task_id,
            tasks: self
                .snapshot
                .tasks
                .into_iter()
                .map(|task| task.map(CliTaskRecord::from))
                .collect(),
            pending_signals: self.snapshot.pending_signals.into_iter().collect(),
        };
        serde_json::to_string_pretty(&snapshot)
            .map_err(|error| format!("cli runtime snapshot encode failed: {error}"))
    }

    pub fn load_snapshot(&mut self, text: &str) -> Result<(), String> {
        let envelope: SnapshotEnvelope = serde_json::from_str(text)
            .map_err(|error| format!("snapshot decode failed: {error}"))?;
        let catalog = envelope.script.compile().map_err(|error| {
            format!("snapshot script compile failed: {}", compile_error_label(error))
        })?;
        self.preset = envelope.preset;
        self.custom_script_loaded = envelope.custom_script_loaded;
        self.script = envelope.script;
        self.catalog = catalog;
        self.snapshot = envelope.runtime;
        self.host = ShowcaseHost {
            active_mind: MindId(envelope.active_mind),
            actions: envelope
                .actions
                .into_iter()
                .map(|action| ShowcaseAction {
                    task_id: action.task_id,
                    action_id: action.action_id,
                    label: action.label,
                })
                .collect(),
            boss_vulnerable: envelope.boss_vulnerable,
        };
        self.trace.clear();
        Ok(())
    }

    pub fn view(&self) -> ShowcaseView {
        ShowcaseView {
            preset: self.preset.as_str().to_owned(),
            title: title_for_preset(self.preset).to_owned(),
            subtitle: subtitle_for_preset(self.preset).to_owned(),
            clock: self.snapshot.clock,
            beat: self.beat_line(),
            active_mind: self.host.active_mind.0,
            boss_vulnerable: self.host.boss_vulnerable,
            tasks: self.collect_tasks(),
            actions: self.host.actions.clone(),
            trace: self.trace.render(),
            snapshot_hint: snapshot_hint_for_preset(self.preset).to_owned(),
        }
    }

    pub fn view_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.view()).map_err(|error| format!("view encode failed: {error}"))
    }

    fn reset_runtime(&mut self) -> Result<(), String> {
        self.snapshot = RuntimeSnapshot::default();
        self.host = ShowcaseHost { active_mind: MindId(1), ..Default::default() };
        self.trace.clear();
        self.spawn_root()
    }

    fn spawn_root(&mut self) -> Result<(), String> {
        self.with_runtime(|runtime, _host, trace| {
            runtime
                .spawn_traced(MAIN_ID, trace)
                .map(|_| ())
                .map_err(|error| format!("spawn failed: {error:?}"))
        })
    }

    fn with_runtime<R>(
        &mut self,
        f: impl for<'a> FnOnce(
            &mut Runtime<'a, MAX_TASKS, MAX_PENDING_SIGNALS>,
            &mut ShowcaseHost,
            &mut TraceLog,
        ) -> Result<R, String>,
    ) -> Result<R, String> {
        let snapshot = &mut self.snapshot;
        let host = &mut self.host;
        let trace = &mut self.trace;
        self.catalog.with_catalog(|catalog| {
            let mut runtime = Runtime::from_snapshot(catalog, *snapshot);
            let result = f(&mut runtime, host, trace);
            *snapshot = runtime.snapshot();
            result
        })
    }

    fn collect_tasks(&self) -> Vec<ShowcaseTask> {
        let preset = self.preset;
        let mut tasks: Vec<ShowcaseTask> = self
            .snapshot
            .tasks
            .into_iter()
            .flatten()
            .map(|task| showcase_task(task, preset))
            .collect();
        tasks.sort_by_key(|task| task.task_id);
        tasks
    }

    fn beat_line(&self) -> String {
        if let Some(root) = self.snapshot_task(TaskId(1)) {
            if root.mind_id != self.host.active_mind {
                return match self.preset {
                    ShowcasePreset::MultiMind => format!(
                        "The {} is parked until the host activates it.",
                        mind_label(self.preset, root.mind_id)
                    ),
                    _ => format!(
                        "Mind {} is parked until mind {} becomes active.",
                        root.mind_id.0, root.mind_id.0
                    ),
                };
            }
        }

        if let Some(action) = self.host.actions.last() {
            return action.label.clone();
        }

        if let Some(root) = self.snapshot_task(TaskId(1)) {
            return match root.wait {
                WaitReason::Ready => "Root is ready to advance.".to_owned(),
                WaitReason::Signal(signal) => {
                    format!("Awaiting signal: {}", signal_label(self.preset, signal))
                }
                WaitReason::Predicate(predicate) => {
                    format!("Awaiting predicate: {}", predicate_label(self.preset, predicate))
                }
                WaitReason::SignalOrTicks { signal, until_tick } => {
                    format!(
                        "Awaiting {} or timeout at clock {}.",
                        signal_label(self.preset, signal),
                        until_tick
                    )
                }
                WaitReason::RaceOrTicks { .. } => match self.preset {
                    ShowcasePreset::DeathOfTick => {
                        "Two branches are racing against the clock. Resolve one or let time decide."
                            .to_owned()
                    }
                    ShowcasePreset::ShootemupBoss => {
                        "The core break and enrage window are racing against the clock.".to_owned()
                    }
                    ShowcasePreset::MultiMind => {
                        "Two branches are racing against the clock.".to_owned()
                    }
                },
                WaitReason::Timeout { child, until_tick } => {
                    format!("Child task {} must finish before clock {}.", child.0, until_tick)
                }
                WaitReason::RepeatUntilPredicate { predicate, .. } => {
                    format!(
                        "Repeating until predicate is ready: {}",
                        predicate_label(self.preset, predicate)
                    )
                }
                WaitReason::ChildrenAll => "Concurrent routines are still resolving.".to_owned(),
                WaitReason::ChildrenAny => {
                    "Waiting for the first concurrent routine to resolve.".to_owned()
                }
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

    fn snapshot_task(&self, task_id: TaskId) -> Option<TaskRecord> {
        self.snapshot.tasks.into_iter().flatten().find(|task| task.id == task_id)
    }

    fn load_builtin_preset(&mut self, preset: ShowcasePreset) -> Result<(), String> {
        let script = script_document_for_preset(preset);
        let catalog = script
            .compile()
            .map_err(|error| format!("default script compile failed: {error:?}"))?;
        self.preset = preset;
        self.custom_script_loaded = false;
        self.script = script;
        self.catalog = catalog;
        self.reset_runtime()
    }
}

fn script_document_for_preset(preset: ShowcasePreset) -> ProgramCatalogDocument {
    match preset {
        ShowcasePreset::DeathOfTick => death_of_tick_script_document(),
        ShowcasePreset::ShootemupBoss => shootemup_boss_script_document(),
        ShowcasePreset::MultiMind => multimind_script_document(),
    }
}

fn death_of_tick_script_document() -> ProgramCatalogDocument {
    ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: MAIN_ID,
                ops: vec![
                    OpDocument::Action { action: ActionId(1) },
                    OpDocument::WaitSignal { signal: PLAYER_COMMITTED },
                    OpDocument::Spawn { program: GATE_ID },
                    OpDocument::Spawn { program: SCOUTS_ID },
                    OpDocument::JoinChildren,
                    OpDocument::WaitPredicate { predicate: BOSS_VULNERABLE },
                    OpDocument::RaceChildren { left: BOSS_INTRO_ID, right: ESCAPE_ID },
                    OpDocument::Action { action: ActionId(6) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: GATE_ID,
                ops: vec![
                    OpDocument::WaitTicks { ticks: 2 },
                    OpDocument::Action { action: ActionId(2) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: SCOUTS_ID,
                ops: vec![
                    OpDocument::WaitSignal { signal: SCOUTS_READY },
                    OpDocument::Action { action: ActionId(3) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: BOSS_INTRO_ID,
                ops: vec![
                    OpDocument::WaitSignal { signal: BOSS_SPOTTED },
                    OpDocument::Action { action: ActionId(4) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ESCAPE_ID,
                ops: vec![
                    OpDocument::WaitSignal { signal: COLLAPSE_TRIGGERED },
                    OpDocument::Action { action: ActionId(5) },
                    OpDocument::Succeed,
                ],
            },
        ],
    }
}

fn shootemup_boss_script_document() -> ProgramCatalogDocument {
    ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: MAIN_ID,
                ops: vec![
                    OpDocument::Action { action: ActionId(101) },
                    OpDocument::WaitSignal { signal: ENGAGE_BOSS_PHASE },
                    OpDocument::Spawn { program: PHASE_WINDUP_ID },
                    OpDocument::Spawn { program: PATTERN_WINDOW_ID },
                    OpDocument::JoinChildren,
                    OpDocument::WaitPredicate { predicate: CORE_EXPOSED },
                    OpDocument::RaceChildrenUntilTick {
                        left: CORE_BREAK_ID,
                        right: ENRAGE_ID,
                        until_tick: 9,
                    },
                    OpDocument::Action { action: ActionId(106) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: PHASE_WINDUP_ID,
                ops: vec![
                    OpDocument::WaitTicks { ticks: 1 },
                    OpDocument::Call {
                        call: HostCallId(1),
                        arg0: 90,
                        arg1: 128,
                        arg2: 16,
                        arg3: 2,
                    },
                    OpDocument::Action { action: ActionId(102) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: PATTERN_WINDOW_ID,
                ops: vec![
                    OpDocument::WaitSignalUntilTick { signal: WING_DRONES_CLEARED, until_tick: 5 },
                    OpDocument::RaceChildrenUntilTick {
                        left: LEFT_SWEEP_ID,
                        right: RIGHT_SWEEP_ID,
                        until_tick: 6,
                    },
                    OpDocument::Action { action: ActionId(103) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: LEFT_SWEEP_ID,
                ops: vec![
                    OpDocument::RepeatCount { count: 4, program: LEFT_BURST_ID },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: RIGHT_SWEEP_ID,
                ops: vec![
                    OpDocument::RepeatCount { count: 4, program: RIGHT_BURST_ID },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: LEFT_BURST_ID,
                ops: vec![
                    OpDocument::Call { call: HostCallId(1), arg0: 1, arg1: 64, arg2: 28, arg3: 5 },
                    OpDocument::WaitTicks { ticks: 1 },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: RIGHT_BURST_ID,
                ops: vec![
                    OpDocument::WaitTicks { ticks: 1 },
                    OpDocument::Call { call: HostCallId(1), arg0: 2, arg1: 192, arg2: 28, arg3: 6 },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: CORE_BREAK_ID,
                ops: vec![
                    OpDocument::WaitSignalUntilTick { signal: BREAK_BOSS_CORE, until_tick: 9 },
                    OpDocument::Action { action: ActionId(104) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument {
                id: ENRAGE_ID,
                ops: vec![
                    OpDocument::WaitUntilTick { until_tick: 8 },
                    OpDocument::Action { action: ActionId(105) },
                    OpDocument::Succeed,
                ],
            },
        ],
    }
}

fn multimind_script_document() -> ProgramCatalogDocument {
    ProgramCatalogDocument {
        programs: vec![
            ProgramDocument {
                id: MAIN_ID,
                ops: vec![
                    OpDocument::Action { action: ActionId(201) },
                    OpDocument::WaitSignal { signal: DIRECTOR_CUE },
                    OpDocument::ChangeMind { mind: GAMEPLAY_MIND },
                    OpDocument::Action { action: ActionId(202) },
                    OpDocument::WaitSignal { signal: GAMEPLAY_BEAT_LOCKED },
                    OpDocument::ChangeMind { mind: DIRECTOR_MIND },
                    OpDocument::Action { action: ActionId(203) },
                    OpDocument::Succeed,
                ],
            },
            ProgramDocument { id: MULTIMIND_STAGE_ID, ops: vec![OpDocument::Succeed] },
        ],
    }
}

fn title_for_preset(preset: ShowcasePreset) -> &'static str {
    match preset {
        ShowcasePreset::DeathOfTick => "The Death of Tick",
        ShowcasePreset::ShootemupBoss => "Shootemup Boss",
        ShowcasePreset::MultiMind => "Mind the Gap",
    }
}

fn subtitle_for_preset(preset: ShowcasePreset) -> &'static str {
    match preset {
        ShowcasePreset::DeathOfTick => {
            "A switchyard showcase for structured encounter orchestration."
        }
        ShowcasePreset::ShootemupBoss => {
            "A switchyard showcase for boss-phase timing, projectile calls, and phase races."
        }
        ShowcasePreset::MultiMind => {
            "A switchyard showcase for director and gameplay minds handing control back and forth."
        }
    }
}

fn snapshot_hint_for_preset(preset: ShowcasePreset) -> &'static str {
    match preset {
        ShowcasePreset::DeathOfTick => {
            "Save a snapshot before resolving the race, then restore and choose the other branch."
        }
        ShowcasePreset::ShootemupBoss => {
            "Save a snapshot after the core is exposed, then either trigger a core break or let enrage win."
        }
        ShowcasePreset::MultiMind => {
            "Save a snapshot after the handoff parks on the gameplay mind, then restore and resume the transfer."
        }
    }
}

fn compile_error_label(error: CompileError) -> String {
    match error {
        CompileError::EmptyCatalog => "catalog is empty".to_owned(),
        CompileError::EmptyProgram { program_id } => {
            format!("program {} has no ops", program_id.0)
        }
        CompileError::DuplicateProgramId(program_id) => {
            format!("duplicate program id {}", program_id.0)
        }
        CompileError::UnknownProgramReference { owner, target } => {
            format!("program {} references missing program {}", owner.0, target.0)
        }
    }
}

fn showcase_task(task: TaskRecord, preset: ShowcasePreset) -> ShowcaseTask {
    ShowcaseTask {
        task_id: task.id.0,
        program_id: task.program_id.0,
        mind: task.mind_id.0,
        program_label: program_label(preset, task.program_id),
        parent: task.parent.map(|parent| parent.0),
        ip: task.ip,
        outcome: outcome_label(task.outcome),
        wait: wait_reason_label(task.wait, preset),
    }
}

fn wait_reason_label(reason: WaitReason, preset: ShowcasePreset) -> String {
    match reason {
        WaitReason::Ready => "ready".to_owned(),
        WaitReason::Ticks { until_tick } => format!("ticks until {until_tick}"),
        WaitReason::SignalOrTicks { signal, until_tick } => {
            format!("signal_or_ticks: {} or clock {}", signal_label(preset, signal), until_tick)
        }
        WaitReason::RaceOrTicks { left, right, until_tick } => {
            format!(
                "race_children_or_ticks: task {} vs task {} until clock {}",
                left.0, right.0, until_tick
            )
        }
        WaitReason::Timeout { child, until_tick } => {
            format!("timeout_ticks: child {} until clock {}", child.0, until_tick)
        }
        WaitReason::Signal(signal) => format!("signal: {}", signal_label(preset, signal)),
        WaitReason::Predicate(predicate) => {
            format!("predicate: {}", predicate_label(preset, predicate))
        }
        WaitReason::RepeatUntilPredicate { predicate, resume_at_tick } => format!(
            "repeat_until_predicate: {} (resume at clock {})",
            predicate_label(preset, predicate),
            resume_at_tick
        ),
        WaitReason::ChildrenAll => "join_children".to_owned(),
        WaitReason::ChildrenAny => "join_any_children".to_owned(),
        WaitReason::Race { left, right } => {
            format!("race between task {} and task {}", left.0, right.0)
        }
    }
}

fn program_label(preset: ShowcasePreset, program_id: ProgramId) -> String {
    match program_id {
        MAIN_ID => match preset {
            ShowcasePreset::DeathOfTick => "main encounter".to_owned(),
            ShowcasePreset::ShootemupBoss => "boss phase".to_owned(),
            ShowcasePreset::MultiMind => "director track".to_owned(),
        },
        GATE_ID => "gate rise".to_owned(),
        SCOUTS_ID => "scout relay".to_owned(),
        BOSS_INTRO_ID => "boss entrance".to_owned(),
        ESCAPE_ID => "collapse escape".to_owned(),
        PHASE_WINDUP_ID => "boss windup".to_owned(),
        PATTERN_WINDOW_ID => "pattern window".to_owned(),
        CORE_BREAK_ID => "core break".to_owned(),
        ENRAGE_ID => "enrage trigger".to_owned(),
        LEFT_SWEEP_ID => "left sweep".to_owned(),
        RIGHT_SWEEP_ID => "right sweep".to_owned(),
        LEFT_BURST_ID => "left burst".to_owned(),
        RIGHT_BURST_ID => "right burst".to_owned(),
        MULTIMIND_STAGE_ID => "stage relay".to_owned(),
        _ => format!("program {}", program_id.0),
    }
}

fn signal_label(preset: ShowcasePreset, signal: SignalId) -> String {
    match preset {
        ShowcasePreset::DeathOfTick => match signal {
            PLAYER_COMMITTED => "player committed".to_owned(),
            SCOUTS_READY => "scouts ready".to_owned(),
            BOSS_SPOTTED => "boss spotted".to_owned(),
            COLLAPSE_TRIGGERED => "collapse triggered".to_owned(),
            _ => format!("signal {}", signal.0),
        },
        ShowcasePreset::ShootemupBoss => match signal {
            ENGAGE_BOSS_PHASE => "wave started".to_owned(),
            WING_DRONES_CLEARED => "wing drones cleared".to_owned(),
            BREAK_BOSS_CORE => "core broken".to_owned(),
            BOMB_TRIGGERED => "bomb triggered".to_owned(),
            _ => format!("signal {}", signal.0),
        },
        ShowcasePreset::MultiMind => match signal {
            DIRECTOR_CUE => "director cue".to_owned(),
            GAMEPLAY_BEAT_LOCKED => "gameplay beat locked".to_owned(),
            SignalId(3) => "cleanup cue".to_owned(),
            SignalId(4) => "final blackout".to_owned(),
            _ => format!("signal {}", signal.0),
        },
    }
}

fn predicate_label(preset: ShowcasePreset, predicate: PredicateId) -> String {
    match preset {
        ShowcasePreset::DeathOfTick if predicate == BOSS_VULNERABLE => "boss vulnerable".to_owned(),
        ShowcasePreset::ShootemupBoss if predicate == CORE_EXPOSED => "core exposed".to_owned(),
        _ => format!("predicate {}", predicate.0),
    }
}

fn mind_label(preset: ShowcasePreset, mind: MindId) -> String {
    match preset {
        ShowcasePreset::MultiMind => match mind {
            DIRECTOR_MIND => "director mind".to_owned(),
            GAMEPLAY_MIND => "gameplay mind".to_owned(),
            _ => format!("mind {}", mind.0),
        },
        _ => format!("mind {}", mind.0),
    }
}

fn action_label(action: ActionId) -> String {
    match action {
        ActionId(1) => "Camera crawls through the switchyard ruins.".to_owned(),
        ActionId(2) => "The portcullis rises on cue.".to_owned(),
        ActionId(3) => "A scout throws the all-clear from the gantry.".to_owned(),
        ActionId(4) => "The boss steps into the lantern light.".to_owned(),
        ActionId(5) => "The emergency escape route tears open.".to_owned(),
        ActionId(6) => "The encounter resolves without a monolithic tick function.".to_owned(),
        ActionId(101) => "The boss frame drifts into the lane and charges the arena.".to_owned(),
        ActionId(102) => "The shutters peel back and the first barrage starts.".to_owned(),
        ActionId(103) => "The opening bullet curtain burns out on schedule.".to_owned(),
        ActionId(104) => "The player cracks the exposed core before enrage.".to_owned(),
        ActionId(105) => "The boss hits enrage and dives into a desperate pattern.".to_owned(),
        ActionId(106) => "The boss phase resolves without a handwritten state machine.".to_owned(),
        ActionId(201) => "The director marks the next scene and prepares the handoff.".to_owned(),
        ActionId(202) => "The gameplay mind takes control and starts the playable beat.".to_owned(),
        ActionId(203) => "The director mind regains control for cleanup and rollout.".to_owned(),
        _ => format!("Action {} emitted.", action.0),
    }
}

fn call_label(call: HostCall) -> String {
    match call.id {
        HostCallId(1) => format!(
            "Spawn projectile pattern={} x={} y={} speed={}.",
            call.args[0], call.args[1], call.args[2], call.args[3]
        ),
        HostCallId(2) => format!(
            "Debug print code={} arg1={} arg2={} arg3={}.",
            call.args[0], call.args[1], call.args[2], call.args[3]
        ),
        _ => format!(
            "Host call {} args=[{}, {}, {}, {}].",
            call.id.0, call.args[0], call.args[1], call.args[2], call.args[3]
        ),
    }
}

fn outcome_label(outcome: Outcome) -> String {
    match outcome {
        Outcome::Running => "running".to_owned(),
        Outcome::Succeeded => "succeeded".to_owned(),
        Outcome::Failed => "failed".to_owned(),
        Outcome::Cancelled => "cancelled".to_owned(),
    }
}

impl From<TaskRecord> for CliTaskRecord {
    fn from(task: TaskRecord) -> Self {
        Self {
            id: task.id,
            program_id: task.program_id,
            mind_id: task.mind_id,
            ip: task.ip,
            parent: task.parent,
            scope_root: task.scope_root,
            outcome: CliOutcome::from(task.outcome),
            wait: CliWaitReason::from(task.wait),
        }
    }
}

impl From<Outcome> for CliOutcome {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Running => Self::Running,
            Outcome::Succeeded => Self::Succeeded,
            Outcome::Failed => Self::Failed,
            Outcome::Cancelled => Self::Cancelled,
        }
    }
}

impl From<WaitReason> for CliWaitReason {
    fn from(wait: WaitReason) -> Self {
        match wait {
            WaitReason::Ready => Self::Ready,
            WaitReason::Ticks { until_tick } => Self::Ticks { until_tick },
            WaitReason::Signal(signal) => Self::Signal { signal },
            WaitReason::Predicate(predicate) => Self::Predicate { predicate },
            WaitReason::SignalOrTicks { signal, until_tick } => {
                Self::SignalOrTicks { signal, until_tick }
            }
            WaitReason::RaceOrTicks { left, right, until_tick } => {
                Self::RaceOrTicks { left, right, until_tick }
            }
            WaitReason::Timeout { child, until_tick } => Self::Timeout { child, until_tick },
            WaitReason::RepeatUntilPredicate { predicate, resume_at_tick } => {
                Self::RepeatUntilPredicate { predicate, resume_at_tick }
            }
            WaitReason::ChildrenAll => Self::ChildrenAll,
            WaitReason::ChildrenAny => Self::ChildrenAny,
            WaitReason::Race { left, right } => Self::Race { left, right },
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

        pub fn reset_script(&mut self) -> Result<String, JsValue> {
            self.inner.reset_script().map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn script_json(&self) -> Result<String, JsValue> {
            self.inner.script_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn export_cli_catalog_json(&self) -> Result<String, JsValue> {
            self.inner.export_cli_catalog_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn load_script(&mut self, script: &str) -> Result<String, JsValue> {
            self.inner.load_script(script).map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn load_preset(&mut self, preset: &str) -> Result<String, JsValue> {
            self.inner.load_preset(preset).map_err(|error| JsValue::from_str(&error))?;
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn set_preset(&mut self, preset: &str) -> Result<String, JsValue> {
            self.inner.load_preset(preset).map_err(|error| JsValue::from_str(&error))?;
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

        pub fn set_active_mind(&mut self, mind_id: u16) -> Result<String, JsValue> {
            self.inner.set_active_mind(MindId(mind_id));
            self.view_json().map_err(|error| JsValue::from_str(&error))
        }

        pub fn save_snapshot(&self) -> Result<String, JsValue> {
            self.inner.save_snapshot().map_err(|error| JsValue::from_str(&error))
        }

        pub fn export_cli_runtime_snapshot_json(&self) -> Result<String, JsValue> {
            self.inner.export_cli_runtime_snapshot_json().map_err(|error| JsValue::from_str(&error))
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
