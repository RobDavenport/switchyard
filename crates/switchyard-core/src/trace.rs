use crate::ids::{ActionId, MindId, ProgramId, SignalId, TaskId};
use crate::program::HostCall;
use crate::runtime::{Outcome, StepReport, WaitReason};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    TickStarted { clock: u64 },
    TickCompleted { report: StepReport },
    SignalQueued { signal: SignalId },
    TaskSpawned { task: TaskId, program_id: ProgramId, parent: Option<TaskId>, scope_root: TaskId },
    TaskWaiting { task: TaskId, reason: WaitReason },
    TaskWoken { task: TaskId, reason: WaitReason },
    ActionEmitted { task: TaskId, action: ActionId },
    CallEmitted { task: TaskId, call: HostCall },
    TaskMindChanged { task: TaskId, from: MindId, to: MindId },
    TaskFinished { task: TaskId, outcome: Outcome },
}

pub trait TraceSink {
    fn on_event(&mut self, event: TraceEvent);
}

pub(crate) struct NoopTraceSink;

impl TraceSink for NoopTraceSink {
    fn on_event(&mut self, _event: TraceEvent) {}
}
