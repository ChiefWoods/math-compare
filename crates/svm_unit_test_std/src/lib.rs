//! Runtime side of the `svm-unit-test` framework.
//!
//! Re-exports [`svm_test`] and provides:
//!  * [`ensure_test_built`] — called by the macro on first invocation of
//!    each `#[svm_test]`; lazily compiles **just that one test's** SBPF
//!    program and returns the path to its `.so`. Sibling tests aren't
//!    touched unless they're also run.
//!  * [`run`] — loads a compiled ELF into Mollusk and reports CU usage.
//!
//! ```ignore
//! use svm_unit_test::svm_test;
//! use core::hint::black_box;
//! use my_lib::{Curve, SCALAR_A, SCALAR_B};
//!
//! #[svm_test]
//! fn add_mod_n() {
//!     black_box(Curve::add_mod_n(black_box(&SCALAR_A), black_box(&SCALAR_B)));
//! }
//! ```

pub use svm_unit_test_macros::svm_test;

pub(crate) mod builder;
mod suite;
pub use suite::ensure_test_built;

use mollusk_svm::{Mollusk, program::loader_keys::LOADER_V3, result::ProgramResult};
use solana_address::Address;
use solana_instruction::Instruction;

/// Re-exported so test files can write `#[svm_test(error = ProgramError::…)]`
/// expectations with a single `use svm_unit_test::ProgramError;` — which the
/// suite parser strips from the SBPF crate (it only keeps non-`svm_unit_test`
/// `use`s), so it never leaks into the on-chain build.
pub use solana_program_error::ProgramError;

#[derive(Debug, Clone)]
pub struct RunReport {
    pub name: String,
    pub compute_units: u64,
    pub execution_time_us: u64,
}

/// What a `#[svm_test]` expects the program's result to be.
#[derive(Debug, Clone)]
pub enum Expect {
    /// The program must execute successfully (the default, plain `#[svm_test]`).
    Success,
    /// The program must *not* succeed — any error is accepted
    /// (`#[svm_test(fail)]` or bare `#[svm_test(error)]`).
    Failure,
    /// The program must fail with exactly this `ProgramError`
    /// (`#[svm_test(error = ProgramError::Custom(1))]`).
    Error(ProgramError),
}

/// Load `elf` into a fresh Mollusk instance, invoke the entrypoint with empty
/// instruction data, and report compute units consumed. Asserts the program
/// succeeds — a thin wrapper over [`run_expecting`] with [`Expect::Success`];
/// panics on program failure.
pub fn run(name: &str, elf: &[u8]) -> RunReport {
    run_expecting(name, elf, Expect::Success)
}

/// Like [`run`], but assert the program's outcome matches `expect`. Panics
/// (failing the `#[test]`) when the actual result diverges from the
/// expectation.
pub fn run_expecting(name: &str, elf: &[u8], expect: Expect) -> RunReport {
    let program_id = Address::new_unique();

    let mut mollusk = Mollusk::default();
    mollusk.add_program_with_loader_and_elf(&program_id, &LOADER_V3, elf);

    let instruction = Instruction::new_with_bytes(program_id, &[], vec![]);
    let result = mollusk.process_instruction(&instruction, &[]);

    match expect {
        Expect::Success => {
            if !result.program_result.is_ok() {
                panic!(
                    "svm_test `{name}` program failed: {:?} (CUs consumed before failure: {})",
                    result.program_result, result.compute_units_consumed,
                );
            }
        }
        Expect::Failure => {
            if result.program_result.is_ok() {
                panic!(
                    "svm_test `{name}` expected the program to fail, but it succeeded (CUs consumed: {})",
                    result.compute_units_consumed,
                );
            }
        }
        Expect::Error(expected) => match &result.program_result {
            ProgramResult::Failure(got) if *got == expected => {}
            other => panic!(
                "svm_test `{name}` expected program error {expected:?}, but got {other:?} (CUs consumed: {})",
                result.compute_units_consumed,
            ),
        },
    }

    let report = RunReport {
        name: name.to_string(),
        compute_units: result.compute_units_consumed,
        execution_time_us: result.execution_time,
    };

    eprintln!(
        "svm_test `{}` => {} CUs ({}us)",
        report.name, report.compute_units, report.execution_time_us,
    );

    report
}
