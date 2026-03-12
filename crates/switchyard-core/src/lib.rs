#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod authoring;
pub mod ids;
pub mod program;
pub mod runtime;
pub mod snapshot;
pub mod trace;

#[cfg(feature = "alloc")]
pub use authoring::{
    CompileError, OpDocument, OwnedProgramCatalog, ProgramCatalogDocument, ProgramDocument,
};
pub use ids::{ActionId, HostCallId, MindId, PredicateId, ProgramId, SignalId, TaskId};
#[cfg(feature = "alloc")]
pub use program::OwnedProgram;
pub use program::{BuildError, HostCall, Op, Program, ProgramBuilder, ProgramCatalog};
pub use runtime::{Host, Outcome, Runtime, RuntimeError, StepReport, TaskRecord, WaitReason};
pub use snapshot::RuntimeSnapshot;
pub use trace::{TraceEvent, TraceSink};
