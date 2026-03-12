use crate::ids::{ActionId, HostCallId, MindId, PredicateId, ProgramId, SignalId};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostCall {
    pub id: HostCallId,
    pub args: [i32; 4],
}

impl HostCall {
    pub const fn new(id: HostCallId, args: [i32; 4]) -> Self {
        Self { id, args }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Action(ActionId),
    Call(HostCall),
    ChangeMind(MindId),
    RepeatUntilPredicate(PredicateId, ProgramId),
    WaitUntilTick(u64),
    WaitSignalUntilTick(SignalId, u64),
    TimeoutUntilTick(u64, ProgramId),
    RaceChildrenUntilTick(ProgramId, ProgramId, u64),
    RaceChildrenOrTicks(ProgramId, ProgramId, u32),
    TimeoutTicks(u32, ProgramId),
    WaitSignalOrTicks(SignalId, u32),
    WaitTicks(u32),
    WaitSignal(SignalId),
    WaitPredicate(PredicateId),
    Spawn(ProgramId),
    BranchPredicate(PredicateId, ProgramId, ProgramId),
    JoinChildren,
    JoinAnyChildren,
    Race2(ProgramId, ProgramId),
    Succeed,
    Fail,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildError {
    CapacityExceeded,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Copy, Debug)]
pub struct Program<'a> {
    pub id: ProgramId,
    pub ops: &'a [Op],
}

impl<'a> Program<'a> {
    pub const fn new(id: ProgramId, ops: &'a [Op]) -> Self {
        Self { id, ops }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProgramBuilder<const CAPACITY: usize> {
    id: ProgramId,
    ops: [Op; CAPACITY],
    len: usize,
}

impl<const CAPACITY: usize> ProgramBuilder<CAPACITY> {
    pub fn new(id: ProgramId) -> Self {
        Self { id, ops: [Op::Succeed; CAPACITY], len: 0 }
    }

    pub fn push(&mut self, op: Op) -> Result<&mut Self, BuildError> {
        if self.len >= CAPACITY {
            return Err(BuildError::CapacityExceeded);
        }

        self.ops[self.len] = op;
        self.len += 1;
        Ok(self)
    }

    pub fn action(&mut self, action: ActionId) -> Result<&mut Self, BuildError> {
        self.push(Op::Action(action))
    }

    pub fn call(&mut self, call_id: HostCallId, args: [i32; 4]) -> Result<&mut Self, BuildError> {
        self.push(Op::Call(HostCall::new(call_id, args)))
    }

    pub fn change_mind(&mut self, mind_id: MindId) -> Result<&mut Self, BuildError> {
        self.push(Op::ChangeMind(mind_id))
    }

    pub fn repeat_until_predicate(
        &mut self,
        predicate: PredicateId,
        program_id: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::RepeatUntilPredicate(predicate, program_id))
    }

    pub fn wait_until_tick(&mut self, until_tick: u64) -> Result<&mut Self, BuildError> {
        self.push(Op::WaitUntilTick(until_tick))
    }

    pub fn wait_signal_until_tick(
        &mut self,
        signal: SignalId,
        until_tick: u64,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::WaitSignalUntilTick(signal, until_tick))
    }

    pub fn timeout_until_tick(
        &mut self,
        until_tick: u64,
        program_id: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::TimeoutUntilTick(until_tick, program_id))
    }

    pub fn race_children_until_tick(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
        until_tick: u64,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::RaceChildrenUntilTick(left_program, right_program, until_tick))
    }

    pub fn race_children_or_ticks(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
        ticks: u32,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::RaceChildrenOrTicks(left_program, right_program, ticks))
    }

    pub fn timeout_ticks(
        &mut self,
        ticks: u32,
        program_id: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::TimeoutTicks(ticks, program_id))
    }

    pub fn wait_signal_or_ticks(
        &mut self,
        signal: SignalId,
        ticks: u32,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::WaitSignalOrTicks(signal, ticks))
    }

    pub fn wait_ticks(&mut self, ticks: u32) -> Result<&mut Self, BuildError> {
        self.push(Op::WaitTicks(ticks))
    }

    pub fn wait_signal(&mut self, signal: SignalId) -> Result<&mut Self, BuildError> {
        self.push(Op::WaitSignal(signal))
    }

    pub fn wait_predicate(&mut self, predicate: PredicateId) -> Result<&mut Self, BuildError> {
        self.push(Op::WaitPredicate(predicate))
    }

    pub fn spawn(&mut self, program_id: ProgramId) -> Result<&mut Self, BuildError> {
        self.push(Op::Spawn(program_id))
    }

    pub fn branch_predicate(
        &mut self,
        predicate: PredicateId,
        if_true: ProgramId,
        if_false: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::BranchPredicate(predicate, if_true, if_false))
    }

    pub fn repeat_count(
        &mut self,
        count: u32,
        program_id: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        let mut remaining = 0u32;
        while remaining < count {
            self.spawn(program_id)?;
            self.join_children()?;
            remaining += 1;
        }
        Ok(self)
    }

    pub fn sync_children(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.spawn(left_program)?;
        self.spawn(right_program)?;
        self.join_children()
    }

    pub fn join_children(&mut self) -> Result<&mut Self, BuildError> {
        self.push(Op::JoinChildren)
    }

    pub fn join_any_children(&mut self) -> Result<&mut Self, BuildError> {
        self.push(Op::JoinAnyChildren)
    }

    pub fn race2(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.push(Op::Race2(left_program, right_program))
    }

    pub fn race_children(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
    ) -> Result<&mut Self, BuildError> {
        self.race2(left_program, right_program)
    }

    pub fn succeed(&mut self) -> Result<&mut Self, BuildError> {
        self.push(Op::Succeed)
    }

    pub fn fail(&mut self) -> Result<&mut Self, BuildError> {
        self.push(Op::Fail)
    }

    pub fn program(&self) -> Program<'_> {
        Program::new(self.id, &self.ops[..self.len])
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnedProgram {
    id: ProgramId,
    ops: Vec<Op>,
}

#[cfg(feature = "alloc")]
impl OwnedProgram {
    pub fn new(id: ProgramId) -> Self {
        Self { id, ops: Vec::new() }
    }

    pub fn id(&self) -> ProgramId {
        self.id
    }

    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    pub fn clear(&mut self) -> &mut Self {
        self.ops.clear();
        self
    }

    pub fn push(&mut self, op: Op) -> &mut Self {
        self.ops.push(op);
        self
    }

    pub fn action(&mut self, action: ActionId) -> &mut Self {
        self.push(Op::Action(action))
    }

    pub fn call(&mut self, call_id: HostCallId, args: [i32; 4]) -> &mut Self {
        self.push(Op::Call(HostCall::new(call_id, args)))
    }

    pub fn change_mind(&mut self, mind_id: MindId) -> &mut Self {
        self.push(Op::ChangeMind(mind_id))
    }

    pub fn repeat_until_predicate(
        &mut self,
        predicate: PredicateId,
        program_id: ProgramId,
    ) -> &mut Self {
        self.push(Op::RepeatUntilPredicate(predicate, program_id))
    }

    pub fn wait_until_tick(&mut self, until_tick: u64) -> &mut Self {
        self.push(Op::WaitUntilTick(until_tick))
    }

    pub fn wait_signal_until_tick(&mut self, signal: SignalId, until_tick: u64) -> &mut Self {
        self.push(Op::WaitSignalUntilTick(signal, until_tick))
    }

    pub fn timeout_until_tick(&mut self, until_tick: u64, program_id: ProgramId) -> &mut Self {
        self.push(Op::TimeoutUntilTick(until_tick, program_id))
    }

    pub fn race_children_until_tick(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
        until_tick: u64,
    ) -> &mut Self {
        self.push(Op::RaceChildrenUntilTick(left_program, right_program, until_tick))
    }

    pub fn race_children_or_ticks(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
        ticks: u32,
    ) -> &mut Self {
        self.push(Op::RaceChildrenOrTicks(left_program, right_program, ticks))
    }

    pub fn timeout_ticks(&mut self, ticks: u32, program_id: ProgramId) -> &mut Self {
        self.push(Op::TimeoutTicks(ticks, program_id))
    }

    pub fn wait_signal_or_ticks(&mut self, signal: SignalId, ticks: u32) -> &mut Self {
        self.push(Op::WaitSignalOrTicks(signal, ticks))
    }

    pub fn wait_ticks(&mut self, ticks: u32) -> &mut Self {
        self.push(Op::WaitTicks(ticks))
    }

    pub fn wait_signal(&mut self, signal: SignalId) -> &mut Self {
        self.push(Op::WaitSignal(signal))
    }

    pub fn wait_predicate(&mut self, predicate: PredicateId) -> &mut Self {
        self.push(Op::WaitPredicate(predicate))
    }

    pub fn spawn(&mut self, program_id: ProgramId) -> &mut Self {
        self.push(Op::Spawn(program_id))
    }

    pub fn branch_predicate(
        &mut self,
        predicate: PredicateId,
        if_true: ProgramId,
        if_false: ProgramId,
    ) -> &mut Self {
        self.push(Op::BranchPredicate(predicate, if_true, if_false))
    }

    pub fn repeat_count(&mut self, count: u32, program_id: ProgramId) -> &mut Self {
        let mut remaining = 0u32;
        while remaining < count {
            self.spawn(program_id).join_children();
            remaining += 1;
        }
        self
    }

    pub fn sync_children(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
    ) -> &mut Self {
        self.spawn(left_program).spawn(right_program).join_children()
    }

    pub fn join_children(&mut self) -> &mut Self {
        self.push(Op::JoinChildren)
    }

    pub fn join_any_children(&mut self) -> &mut Self {
        self.push(Op::JoinAnyChildren)
    }

    pub fn race2(&mut self, left_program: ProgramId, right_program: ProgramId) -> &mut Self {
        self.push(Op::Race2(left_program, right_program))
    }

    pub fn race_children(
        &mut self,
        left_program: ProgramId,
        right_program: ProgramId,
    ) -> &mut Self {
        self.race2(left_program, right_program)
    }

    pub fn succeed(&mut self) -> &mut Self {
        self.push(Op::Succeed)
    }

    pub fn fail(&mut self) -> &mut Self {
        self.push(Op::Fail)
    }

    pub fn as_program(&self) -> Program<'_> {
        Program::new(self.id, &self.ops)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProgramCatalog<'a> {
    programs: &'a [Program<'a>],
}

impl<'a> ProgramCatalog<'a> {
    pub const fn new(programs: &'a [Program<'a>]) -> Self {
        Self { programs }
    }

    pub fn get(&self, id: ProgramId) -> Option<&'a Program<'a>> {
        let mut index = 0usize;
        while index < self.programs.len() {
            if self.programs[index].id == id {
                return Some(&self.programs[index]);
            }
            index += 1;
        }
        None
    }

    pub const fn programs(&self) -> &'a [Program<'a>] {
        self.programs
    }
}
