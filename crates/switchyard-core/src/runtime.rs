use crate::ids::{ActionId, PredicateId, ProgramId, SignalId, TaskId};
use crate::program::ProgramCatalog;
use crate::snapshot::RuntimeSnapshot;
use crate::trace::{NoopTraceSink, TraceEvent, TraceSink};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WaitReason {
    #[default]
    Ready,
    Ticks {
        until_tick: u64,
    },
    Signal(SignalId),
    Predicate(PredicateId),
    ChildrenAll,
    Race {
        left: TaskId,
        right: TaskId,
    },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskRecord {
    pub id: TaskId,
    pub program_id: ProgramId,
    pub ip: usize,
    pub parent: Option<TaskId>,
    pub scope_root: TaskId,
    pub outcome: Outcome,
    pub wait: WaitReason,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepReport {
    pub clock: u64,
    pub actions_emitted: usize,
    pub progress_made: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    ProgramNotFound(ProgramId),
    CapacityExceeded,
    SignalQueueFull,
}

pub trait Host {
    fn on_action(&mut self, task: TaskId, action: ActionId);

    fn query_ready(&mut self, _predicate: PredicateId) -> bool {
        false
    }
}

pub struct Runtime<'a, const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize> {
    programs: ProgramCatalog<'a>,
    snapshot: RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>,
}

impl<'a, const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize>
    Runtime<'a, MAX_TASKS, MAX_PENDING_SIGNALS>
{
    pub fn new(programs: ProgramCatalog<'a>) -> Self {
        Self { programs, snapshot: RuntimeSnapshot::default() }
    }

    pub fn from_snapshot(
        programs: ProgramCatalog<'a>,
        snapshot: RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>,
    ) -> Self {
        Self { programs, snapshot }
    }

    pub fn snapshot(&self) -> RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS> {
        self.snapshot
    }

    pub const fn clock(&self) -> u64 {
        self.snapshot.clock
    }

    pub fn task(&self, task_id: TaskId) -> Option<TaskRecord> {
        let mut index = 0usize;
        while index < MAX_TASKS {
            if let Some(task) = self.snapshot.tasks[index] {
                if task.id == task_id {
                    return Some(task);
                }
            }
            index += 1;
        }
        None
    }

    pub fn tasks(&self) -> [Option<TaskRecord>; MAX_TASKS] {
        self.snapshot.tasks
    }

    pub fn spawn(&mut self, program_id: ProgramId) -> Result<TaskId, RuntimeError> {
        let mut trace = NoopTraceSink;
        self.spawn_traced(program_id, &mut trace)
    }

    pub fn spawn_traced<T: TraceSink>(
        &mut self,
        program_id: ProgramId,
        trace: &mut T,
    ) -> Result<TaskId, RuntimeError> {
        self.spawn_task(program_id, None, None, trace)
    }

    pub fn emit_signal(&mut self, signal: SignalId) -> Result<(), RuntimeError> {
        let mut trace = NoopTraceSink;
        self.emit_signal_traced(signal, &mut trace)
    }

    pub fn emit_signal_traced<T: TraceSink>(
        &mut self,
        signal: SignalId,
        trace: &mut T,
    ) -> Result<(), RuntimeError> {
        let mut index = 0usize;
        while index < MAX_PENDING_SIGNALS {
            if self.snapshot.pending_signals[index].is_none() {
                self.snapshot.pending_signals[index] = Some(signal);
                trace.on_event(TraceEvent::SignalQueued { signal });
                return Ok(());
            }
            index += 1;
        }
        Err(RuntimeError::SignalQueueFull)
    }

    pub fn tick<H: Host>(&mut self, host: &mut H) -> Result<StepReport, RuntimeError> {
        let mut trace = NoopTraceSink;
        self.tick_traced(host, &mut trace)
    }

    pub fn tick_traced<H: Host, T: TraceSink>(
        &mut self,
        host: &mut H,
        trace: &mut T,
    ) -> Result<StepReport, RuntimeError> {
        self.snapshot.clock = self.snapshot.clock.saturating_add(1);
        let mut report =
            StepReport { clock: self.snapshot.clock, actions_emitted: 0, progress_made: false };
        trace.on_event(TraceEvent::TickStarted { clock: self.snapshot.clock });

        loop {
            let ordered = self.ordered_slots();
            let mut round_progress = false;
            let mut cursor = 0usize;

            while cursor < MAX_TASKS {
                if let Some(slot) = ordered[cursor] {
                    if self.wake_task_if_ready(slot, host, trace) {
                        round_progress = true;
                    }
                    if self.run_task(slot, host, &mut report, trace)? {
                        round_progress = true;
                    }
                }
                cursor += 1;
            }

            if !round_progress {
                break;
            }

            report.progress_made = true;
        }

        let mut index = 0usize;
        while index < MAX_PENDING_SIGNALS {
            self.snapshot.pending_signals[index] = None;
            index += 1;
        }

        trace.on_event(TraceEvent::TickCompleted { report });
        Ok(report)
    }

    fn run_task<H: Host, T: TraceSink>(
        &mut self,
        slot: usize,
        host: &mut H,
        report: &mut StepReport,
        trace: &mut T,
    ) -> Result<bool, RuntimeError> {
        let Some(mut task) = self.snapshot.tasks[slot] else {
            return Ok(false);
        };

        if task.outcome != Outcome::Running || task.wait != WaitReason::Ready {
            return Ok(false);
        }

        let mut progress = false;

        loop {
            if task.outcome != Outcome::Running || task.wait != WaitReason::Ready {
                break;
            }

            let op = {
                let Some(program) = self.programs.get(task.program_id) else {
                    return Err(RuntimeError::ProgramNotFound(task.program_id));
                };

                if task.ip >= program.ops.len() {
                    self.snapshot.tasks[slot] = Some(task);
                    self.finish_task(task.id, Outcome::Succeeded, trace);
                    progress = true;
                    break;
                }

                program.ops[task.ip]
            };

            match op {
                crate::program::Op::Action(action) => {
                    host.on_action(task.id, action);
                    trace.on_event(TraceEvent::ActionEmitted { task: task.id, action });
                    report.actions_emitted += 1;
                    task.ip += 1;
                    self.snapshot.tasks[slot] = Some(task);
                    progress = true;
                }
                crate::program::Op::WaitTicks(ticks) => {
                    task.ip += 1;
                    task.wait = WaitReason::Ticks {
                        until_tick: self.snapshot.clock.saturating_add(u64::from(ticks)),
                    };
                    self.snapshot.tasks[slot] = Some(task);
                    trace.on_event(TraceEvent::TaskWaiting { task: task.id, reason: task.wait });
                    progress = true;
                    break;
                }
                crate::program::Op::WaitSignal(signal) => {
                    task.ip += 1;
                    task.wait = WaitReason::Signal(signal);
                    self.snapshot.tasks[slot] = Some(task);
                    trace.on_event(TraceEvent::TaskWaiting { task: task.id, reason: task.wait });
                    progress = true;
                    break;
                }
                crate::program::Op::WaitPredicate(predicate) => {
                    task.ip += 1;
                    task.wait = WaitReason::Predicate(predicate);
                    self.snapshot.tasks[slot] = Some(task);
                    trace.on_event(TraceEvent::TaskWaiting { task: task.id, reason: task.wait });
                    progress = true;
                    break;
                }
                crate::program::Op::Spawn(program_id) => {
                    task.ip += 1;
                    self.snapshot.tasks[slot] = Some(task);
                    let _ =
                        self.spawn_task(program_id, Some(task.id), Some(task.scope_root), trace)?;
                    progress = true;
                }
                crate::program::Op::JoinChildren => {
                    task.ip += 1;
                    if self.has_running_children(task.id) {
                        task.wait = WaitReason::ChildrenAll;
                    }
                    self.snapshot.tasks[slot] = Some(task);
                    if task.wait == WaitReason::ChildrenAll {
                        trace
                            .on_event(TraceEvent::TaskWaiting { task: task.id, reason: task.wait });
                    }
                    progress = true;
                    if self.has_running_children(task.id) {
                        break;
                    }
                }
                crate::program::Op::Race2(left_program, right_program) => {
                    if self.available_slots() < 2 {
                        return Err(RuntimeError::CapacityExceeded);
                    }
                    task.ip += 1;
                    self.snapshot.tasks[slot] = Some(task);
                    let left =
                        self.spawn_task(left_program, Some(task.id), Some(task.scope_root), trace)?;
                    let right = self.spawn_task(
                        right_program,
                        Some(task.id),
                        Some(task.scope_root),
                        trace,
                    )?;
                    let mut current = self.snapshot.tasks[slot].expect("task must exist");
                    current.wait = WaitReason::Race { left, right };
                    self.snapshot.tasks[slot] = Some(current);
                    trace.on_event(TraceEvent::TaskWaiting {
                        task: current.id,
                        reason: current.wait,
                    });
                    progress = true;
                    break;
                }
                crate::program::Op::Succeed => {
                    self.snapshot.tasks[slot] = Some(task);
                    self.finish_task(task.id, Outcome::Succeeded, trace);
                    progress = true;
                    break;
                }
                crate::program::Op::Fail => {
                    self.snapshot.tasks[slot] = Some(task);
                    self.finish_task(task.id, Outcome::Failed, trace);
                    progress = true;
                    break;
                }
            }
        }

        Ok(progress)
    }

    fn wake_task_if_ready<H: Host, T: TraceSink>(
        &mut self,
        slot: usize,
        host: &mut H,
        trace: &mut T,
    ) -> bool {
        let Some(mut task) = self.snapshot.tasks[slot] else {
            return false;
        };

        if task.outcome != Outcome::Running {
            return false;
        }

        let ready = match task.wait {
            WaitReason::Ready => false,
            WaitReason::Ticks { until_tick } => self.snapshot.clock >= until_tick,
            WaitReason::Signal(signal) => self.signal_pending(signal),
            WaitReason::Predicate(predicate) => host.query_ready(predicate),
            WaitReason::ChildrenAll => !self.has_running_children(task.id),
            WaitReason::Race { left, right } => {
                if self.task_outcome(left) != Outcome::Running {
                    self.cancel_if_running(right, trace);
                    true
                } else if self.task_outcome(right) != Outcome::Running {
                    self.cancel_if_running(left, trace);
                    true
                } else {
                    false
                }
            }
        };

        if ready {
            let reason = task.wait;
            task.wait = WaitReason::Ready;
            self.snapshot.tasks[slot] = Some(task);
            trace.on_event(TraceEvent::TaskWoken { task: task.id, reason });
            true
        } else {
            false
        }
    }

    fn spawn_task<T: TraceSink>(
        &mut self,
        program_id: ProgramId,
        parent: Option<TaskId>,
        scope_root: Option<TaskId>,
        trace: &mut T,
    ) -> Result<TaskId, RuntimeError> {
        if self.programs.get(program_id).is_none() {
            return Err(RuntimeError::ProgramNotFound(program_id));
        }

        let Some(slot) = self.find_free_slot() else {
            return Err(RuntimeError::CapacityExceeded);
        };

        self.snapshot.next_task_id = self.snapshot.next_task_id.saturating_add(1);
        let id = TaskId(self.snapshot.next_task_id);
        let root = scope_root.unwrap_or(id);

        self.snapshot.tasks[slot] = Some(TaskRecord {
            id,
            program_id,
            ip: 0,
            parent,
            scope_root: root,
            outcome: Outcome::Running,
            wait: WaitReason::Ready,
        });
        trace.on_event(TraceEvent::TaskSpawned { task: id, program_id, parent, scope_root: root });

        Ok(id)
    }

    fn finish_task<T: TraceSink>(&mut self, task_id: TaskId, outcome: Outcome, trace: &mut T) {
        let Some(slot) = self.find_task_slot(task_id) else {
            return;
        };

        let Some(mut task) = self.snapshot.tasks[slot] else {
            return;
        };

        task.outcome = outcome;
        task.wait = WaitReason::Ready;
        self.snapshot.tasks[slot] = Some(task);
        trace.on_event(TraceEvent::TaskFinished { task: task.id, outcome });

        let children = self.child_ids_of(task_id);
        let mut index = 0usize;
        while index < MAX_TASKS {
            if let Some(child) = children[index] {
                self.cancel_if_running(child, trace);
            }
            index += 1;
        }
    }

    fn cancel_if_running<T: TraceSink>(&mut self, task_id: TaskId, trace: &mut T) {
        let Some(slot) = self.find_task_slot(task_id) else {
            return;
        };

        let Some(task) = self.snapshot.tasks[slot] else {
            return;
        };

        if task.outcome != Outcome::Running {
            return;
        }

        let children = self.child_ids_of(task.id);
        let mut index = 0usize;
        while index < MAX_TASKS {
            if let Some(child) = children[index] {
                self.cancel_if_running(child, trace);
            }
            index += 1;
        }

        let mut cancelled = task;
        cancelled.outcome = Outcome::Cancelled;
        cancelled.wait = WaitReason::Ready;
        self.snapshot.tasks[slot] = Some(cancelled);
        trace
            .on_event(TraceEvent::TaskFinished { task: cancelled.id, outcome: Outcome::Cancelled });
    }

    fn child_ids_of(&self, parent_id: TaskId) -> [Option<TaskId>; MAX_TASKS] {
        let mut out = [None; MAX_TASKS];
        let mut len = 0usize;
        let mut index = 0usize;
        while index < MAX_TASKS {
            if let Some(task) = self.snapshot.tasks[index] {
                if task.parent == Some(parent_id) {
                    out[len] = Some(task.id);
                    len += 1;
                }
            }
            index += 1;
        }
        out
    }

    fn has_running_children(&self, parent_id: TaskId) -> bool {
        let mut index = 0usize;
        while index < MAX_TASKS {
            if let Some(task) = self.snapshot.tasks[index] {
                if task.parent == Some(parent_id) && task.outcome == Outcome::Running {
                    return true;
                }
            }
            index += 1;
        }
        false
    }

    fn signal_pending(&self, signal: SignalId) -> bool {
        let mut index = 0usize;
        while index < MAX_PENDING_SIGNALS {
            if self.snapshot.pending_signals[index] == Some(signal) {
                return true;
            }
            index += 1;
        }
        false
    }

    fn task_outcome(&self, task_id: TaskId) -> Outcome {
        self.task(task_id).map(|task| task.outcome).unwrap_or(Outcome::Cancelled)
    }

    fn available_slots(&self) -> usize {
        let mut count = 0usize;
        let mut index = 0usize;
        while index < MAX_TASKS {
            if self.snapshot.tasks[index].is_none() {
                count += 1;
            }
            index += 1;
        }
        count
    }

    fn find_free_slot(&self) -> Option<usize> {
        let mut index = 0usize;
        while index < MAX_TASKS {
            if self.snapshot.tasks[index].is_none() {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn find_task_slot(&self, task_id: TaskId) -> Option<usize> {
        let mut index = 0usize;
        while index < MAX_TASKS {
            if let Some(task) = self.snapshot.tasks[index] {
                if task.id == task_id {
                    return Some(index);
                }
            }
            index += 1;
        }
        None
    }

    fn ordered_slots(&self) -> [Option<usize>; MAX_TASKS] {
        let mut ordered = [None; MAX_TASKS];
        let mut len = 0usize;

        let mut slot = 0usize;
        while slot < MAX_TASKS {
            if self.snapshot.tasks[slot].is_some() {
                ordered[len] = Some(slot);
                len += 1;
            }
            slot += 1;
        }

        let mut index = 1usize;
        while index < len {
            let current_slot = ordered[index].expect("occupied slot");
            let current_id = self.snapshot.tasks[current_slot].expect("occupied slot").id.0;
            let mut inner = index;
            while inner > 0 {
                let previous_slot = ordered[inner - 1].expect("occupied slot");
                let previous_id = self.snapshot.tasks[previous_slot].expect("occupied slot").id.0;
                if previous_id <= current_id {
                    break;
                }
                ordered[inner] = ordered[inner - 1];
                inner -= 1;
            }
            ordered[inner] = Some(current_slot);
            index += 1;
        }

        ordered
    }
}
