use crate::ids::{ActionId, HostCallId, MindId, PredicateId, ProgramId, SignalId};
use crate::program::{HostCall, Op, OwnedProgram, Program, ProgramCatalog};
use alloc::vec::Vec;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgramCatalogDocument {
    pub programs: Vec<ProgramDocument>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramDocument {
    pub id: ProgramId,
    pub ops: Vec<OpDocument>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "op", rename_all = "snake_case"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpDocument {
    Action { action: ActionId },
    Call { call: HostCallId, arg0: i32, arg1: i32, arg2: i32, arg3: i32 },
    ChangeMind { mind: MindId },
    RepeatUntilPredicate { predicate: PredicateId, program: ProgramId },
    WaitUntilTick { until_tick: u64 },
    WaitSignalUntilTick { signal: SignalId, until_tick: u64 },
    TimeoutUntilTick { until_tick: u64, program: ProgramId },
    RaceChildrenUntilTick { left: ProgramId, right: ProgramId, until_tick: u64 },
    RaceChildrenOrTicks { left: ProgramId, right: ProgramId, ticks: u32 },
    TimeoutTicks { ticks: u32, program: ProgramId },
    WaitSignalOrTicks { signal: SignalId, ticks: u32 },
    WaitTicks { ticks: u32 },
    WaitSignal { signal: SignalId },
    WaitPredicate { predicate: PredicateId },
    Spawn { program: ProgramId },
    RepeatCount { count: u32, program: ProgramId },
    SyncChildren { left: ProgramId, right: ProgramId },
    BranchPredicate { predicate: PredicateId, if_true: ProgramId, if_false: ProgramId },
    JoinChildren,
    JoinAnyChildren,
    RaceChildren { left: ProgramId, right: ProgramId },
    Race2 { left: ProgramId, right: ProgramId },
    Succeed,
    Fail,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnedProgramCatalog {
    programs: Vec<OwnedProgram>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileError {
    EmptyCatalog,
    EmptyProgram { program_id: ProgramId },
    DuplicateProgramId(ProgramId),
    UnknownProgramReference { owner: ProgramId, target: ProgramId },
}

impl ProgramCatalogDocument {
    pub fn compile(&self) -> Result<OwnedProgramCatalog, CompileError> {
        if self.programs.is_empty() {
            return Err(CompileError::EmptyCatalog);
        }

        let mut seen = Vec::with_capacity(self.programs.len());
        for program in &self.programs {
            if program.ops.is_empty() {
                return Err(CompileError::EmptyProgram { program_id: program.id });
            }
            if seen.contains(&program.id) {
                return Err(CompileError::DuplicateProgramId(program.id));
            }
            seen.push(program.id);
        }

        for program in &self.programs {
            for target in program.referenced_programs() {
                if !seen.contains(&target) {
                    return Err(CompileError::UnknownProgramReference {
                        owner: program.id,
                        target,
                    });
                }
            }
        }

        let mut compiled = Vec::with_capacity(self.programs.len());
        for program in &self.programs {
            compiled.push(program.compile());
        }

        Ok(OwnedProgramCatalog::new(compiled))
    }
}

impl ProgramDocument {
    pub fn compile(&self) -> OwnedProgram {
        let mut program = OwnedProgram::new(self.id);
        for op in &self.ops {
            match op {
                OpDocument::RepeatCount { count, program: child } => {
                    program.repeat_count(*count, *child);
                }
                OpDocument::SyncChildren { left, right } => {
                    program.sync_children(*left, *right);
                }
                _ => {
                    program.push(op.compile());
                }
            }
        }
        program
    }

    fn referenced_programs(&self) -> Vec<ProgramId> {
        let mut referenced = Vec::new();
        for op in &self.ops {
            match op {
                OpDocument::Spawn { program } => referenced.push(*program),
                OpDocument::RepeatUntilPredicate { program, .. } => referenced.push(*program),
                OpDocument::TimeoutUntilTick { program, .. } => referenced.push(*program),
                OpDocument::RaceChildrenUntilTick { left, right, .. } => {
                    referenced.push(*left);
                    referenced.push(*right);
                }
                OpDocument::RaceChildrenOrTicks { left, right, .. } => {
                    referenced.push(*left);
                    referenced.push(*right);
                }
                OpDocument::TimeoutTicks { program, .. } => referenced.push(*program),
                OpDocument::RepeatCount { program, .. } => referenced.push(*program),
                OpDocument::SyncChildren { left, right } => {
                    referenced.push(*left);
                    referenced.push(*right);
                }
                OpDocument::BranchPredicate { if_true, if_false, .. } => {
                    referenced.push(*if_true);
                    referenced.push(*if_false);
                }
                OpDocument::RaceChildren { left, right } => {
                    referenced.push(*left);
                    referenced.push(*right);
                }
                OpDocument::Race2 { left, right } => {
                    referenced.push(*left);
                    referenced.push(*right);
                }
                _ => {}
            }
        }
        referenced
    }
}

impl OpDocument {
    pub fn compile(&self) -> Op {
        match self {
            Self::Action { action } => Op::Action(*action),
            Self::Call { call, arg0, arg1, arg2, arg3 } => {
                Op::Call(HostCall::new(*call, [*arg0, *arg1, *arg2, *arg3]))
            }
            Self::ChangeMind { mind } => Op::ChangeMind(*mind),
            Self::RepeatUntilPredicate { predicate, program } => {
                Op::RepeatUntilPredicate(*predicate, *program)
            }
            Self::WaitUntilTick { until_tick } => Op::WaitUntilTick(*until_tick),
            Self::WaitSignalUntilTick { signal, until_tick } => {
                Op::WaitSignalUntilTick(*signal, *until_tick)
            }
            Self::TimeoutUntilTick { until_tick, program } => {
                Op::TimeoutUntilTick(*until_tick, *program)
            }
            Self::RaceChildrenUntilTick { left, right, until_tick } => {
                Op::RaceChildrenUntilTick(*left, *right, *until_tick)
            }
            Self::RaceChildrenOrTicks { left, right, ticks } => {
                Op::RaceChildrenOrTicks(*left, *right, *ticks)
            }
            Self::TimeoutTicks { ticks, program } => Op::TimeoutTicks(*ticks, *program),
            Self::WaitSignalOrTicks { signal, ticks } => Op::WaitSignalOrTicks(*signal, *ticks),
            Self::WaitTicks { ticks } => Op::WaitTicks(*ticks),
            Self::WaitSignal { signal } => Op::WaitSignal(*signal),
            Self::WaitPredicate { predicate } => Op::WaitPredicate(*predicate),
            Self::Spawn { program } => Op::Spawn(*program),
            Self::RepeatCount { .. } => unreachable!("repeat_count lowers during document compile"),
            Self::SyncChildren { .. } => {
                unreachable!("sync_children lowers during document compile")
            }
            Self::BranchPredicate { predicate, if_true, if_false } => {
                Op::BranchPredicate(*predicate, *if_true, *if_false)
            }
            Self::JoinChildren => Op::JoinChildren,
            Self::JoinAnyChildren => Op::JoinAnyChildren,
            Self::RaceChildren { left, right } => Op::Race2(*left, *right),
            Self::Race2 { left, right } => Op::Race2(*left, *right),
            Self::Succeed => Op::Succeed,
            Self::Fail => Op::Fail,
        }
    }
}

impl OwnedProgramCatalog {
    pub fn new(programs: Vec<OwnedProgram>) -> Self {
        Self { programs }
    }

    pub fn programs(&self) -> &[OwnedProgram] {
        &self.programs
    }

    pub fn get(&self, id: ProgramId) -> Option<&OwnedProgram> {
        self.programs.iter().find(|program| program.id() == id)
    }

    pub fn with_catalog<R>(&self, f: impl FnOnce(ProgramCatalog<'_>) -> R) -> R {
        let programs: Vec<Program<'_>> =
            self.programs.iter().map(OwnedProgram::as_program).collect();
        f(ProgramCatalog::new(&programs))
    }
}
