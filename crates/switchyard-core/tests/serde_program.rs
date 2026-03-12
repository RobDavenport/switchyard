#![cfg(all(feature = "alloc", feature = "serde"))]

use switchyard_core::{ActionId, Op, OwnedProgram, ProgramId, SignalId};

#[test]
fn owned_program_round_trips_through_serde_json() {
    let mut program = OwnedProgram::new(ProgramId(9));
    program.wait_signal(SignalId(3)).action(ActionId(5)).succeed();

    let json = serde_json::to_string_pretty(&program).expect("serialize owned program");
    let restored: OwnedProgram = serde_json::from_str(&json).expect("deserialize owned program");

    assert_eq!(restored.id(), ProgramId(9));
    assert_eq!(
        restored.ops(),
        &[Op::WaitSignal(SignalId(3)), Op::Action(ActionId(5)), Op::Succeed]
    );
}
