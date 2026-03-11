use crate::ids::SignalId;
use crate::runtime::TaskRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSnapshot<const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize> {
    pub clock: u64,
    pub next_task_id: u32,
    pub tasks: [Option<TaskRecord>; MAX_TASKS],
    pub pending_signals: [Option<SignalId>; MAX_PENDING_SIGNALS],
}

impl<const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize>
    RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>
{
    pub fn empty() -> Self {
        Self {
            clock: 0,
            next_task_id: 0,
            tasks: core::array::from_fn(|_| None),
            pending_signals: core::array::from_fn(|_| None),
        }
    }
}

impl<const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize> Default
    for RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>
{
    fn default() -> Self {
        Self::empty()
    }
}
