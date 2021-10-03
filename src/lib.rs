use core::{convert::TryInto, ops::Range};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, StarkField},
    ExecutionTrace, ProofOptions, ProverError, Serializable, StarkProof, VerifierError,
};

// RE-EXPORTS
// ================================================================================================
pub mod utils;

mod air;
pub use air::utils::ToElements;
use air::{ProcessorAir, PublicInputs, TraceMetadata, TraceState};

mod processor;
pub use processor::{OpCode, OpHint};

mod programs;
pub use programs::{assembly, blocks, Program, ProgramInputs};

// EXECUTOR
// ================================================================================================

/// Executes the specified `program` and returns the result together with a STARK-based proof of execution.
///
/// * `inputs` specifies the initial stack state and provides secret input tapes;
/// * `num_outputs` specifies the number of elements from the top of the stack to be returned;
pub fn execute(
    program: &Program,
    inputs: &ProgramInputs,
    num_outputs: usize,
    options: &ProofOptions,
) -> Result<(Vec<u128>, StarkProof), ProverError> {
    assert!(
        num_outputs <= MAX_OUTPUTS,
        "cannot produce more than {} outputs, but requested {}",
        MAX_OUTPUTS,
        num_outputs
    );

    // execute the program to create an execution trace
    let now = Instant::now();
    let trace = processor::execute(program, inputs);
    debug!(
        "Generated execution trace of {} registers and {} steps in {} ms",
        trace.width(),
        trace.length(),
        now.elapsed().as_millis()
    );

    // copy the user stack state the the last step to return as output
    let last_state = get_last_state(&trace);
    let outputs = last_state.user_stack()[..num_outputs]
        .iter()
        .map(|&v| v.as_int())
        .collect::<Vec<_>>();

    // make sure number of executed operations was sufficient
    assert!(
        last_state.op_counter().as_int() as usize >= MIN_TRACE_LENGTH,
        "a program must consist of at least {} operation, but only {} were executed",
        MIN_TRACE_LENGTH,
        last_state.op_counter()
    );

    // make sure program hash generated by the VM matches the hash of the program
    let program_hash: [u8; 32] = last_state.program_hash().to_bytes().try_into().unwrap();
    assert!(
        *program.hash() == program_hash,
        "expected program hash {} does not match trace hash {}",
        hex::encode(program.hash()),
        hex::encode(program_hash)
    );

    // generate STARK proof
    let inputs = inputs
        .get_public_inputs()
        .iter()
        .map(|&v| v.as_int())
        .collect::<Vec<_>>();
    let pub_inputs = PublicInputs::new(program_hash, &inputs, &outputs);
    let proof = winterfell::prove::<ProcessorAir>(trace, pub_inputs, options.clone())?;

    Ok((outputs, proof))
}

// VERIFIER
// ================================================================================================

/// Returns Ok(()) if the specified program was executed correctly against the specified inputs
/// and outputs.
///
/// Specifically, verifies that if a program with the specified `program_hash` is executed with the
/// provided `public_inputs` and some secret inputs, and the result is equal to the `outputs`.
///
/// # Errors
/// Returns an error if the provided proof does not prove a correct execution of the program.
pub fn verify(
    program_hash: [u8; 32],
    public_inputs: &[u128],
    outputs: &[u128],
    proof: StarkProof,
) -> Result<(), VerifierError> {
    let pub_inputs = PublicInputs::new(program_hash, public_inputs, outputs);
    winterfell::verify::<ProcessorAir>(proof, pub_inputs)
}

// GLOBAL CONSTANTS
// ================================================================================================

pub const MAX_CONTEXT_DEPTH: usize = 16;
pub const MAX_LOOP_DEPTH: usize = 8;
const MIN_TRACE_LENGTH: usize = 16;
const BASE_CYCLE_LENGTH: usize = 16;

const MIN_STACK_DEPTH: usize = 8;
const MIN_CONTEXT_DEPTH: usize = 1;
const MIN_LOOP_DEPTH: usize = 1;

// PUSH OPERATION
// ------------------------------------------------------------------------------------------------
const PUSH_OP_ALIGNMENT: usize = 8;

// HASH OPERATION
// ------------------------------------------------------------------------------------------------
const HASH_STATE_RATE: usize = 4;
const HASH_STATE_CAPACITY: usize = 2;
const HASH_STATE_WIDTH: usize = HASH_STATE_RATE + HASH_STATE_CAPACITY;
const HASH_NUM_ROUNDS: usize = 10;
const HASH_DIGEST_SIZE: usize = 2;

// OPERATION SPONGE
// ------------------------------------------------------------------------------------------------
const SPONGE_WIDTH: usize = 4;
const PROGRAM_DIGEST_SIZE: usize = 2;
const HACC_NUM_ROUNDS: usize = 14;

// DECODER LAYOUT
// ------------------------------------------------------------------------------------------------
//
//  ctr ╒═════ sponge ══════╕╒═══ cf_ops ══╕╒═══════ ld_ops ═══════╕╒═ hd_ops ╕╒═ ctx ══╕╒═ loop ═╕
//   0    1    2    3    4    5    6    7    8    9    10   11   12   13   14   15   ..   ..   ..
// ├────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┤

const NUM_CF_OP_BITS: usize = 3;
const NUM_LD_OP_BITS: usize = 5;
const NUM_HD_OP_BITS: usize = 2;

const NUM_CF_OPS: usize = 8;
const NUM_LD_OPS: usize = 32;
const NUM_HD_OPS: usize = 4;

const OP_COUNTER_IDX: usize = 0;
const OP_SPONGE_RANGE: Range<usize> = Range { start: 1, end: 5 };
const CF_OP_BITS_RANGE: Range<usize> = Range { start: 5, end: 8 };
const LD_OP_BITS_RANGE: Range<usize> = Range { start: 8, end: 13 };
const HD_OP_BITS_RANGE: Range<usize> = Range { start: 13, end: 15 };

// STACK LAYOUT
// ------------------------------------------------------------------------------------------------
//
// ╒═══════════════════ user registers ════════════════════════╕
//    0      1    2    .................................    31
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤

pub const MAX_PUBLIC_INPUTS: usize = 8;
pub const MAX_OUTPUTS: usize = MAX_PUBLIC_INPUTS;
pub const MAX_STACK_DEPTH: usize = 32;

// HELPER FUNCTIONS
// ================================================================================================

fn get_last_state(trace: &ExecutionTrace<BaseElement>) -> TraceState<BaseElement> {
    let last_step = trace.length() - 1;
    let meta = TraceMetadata::from_trace_info(&trace.get_info());

    let mut last_row = vec![BaseElement::ZERO; trace.width()];
    trace.read_row_into(last_step, &mut last_row);

    TraceState::from_vec(meta.ctx_depth, meta.loop_depth, meta.stack_depth, &last_row)
}

/// Prints out an execution trace.
#[allow(unused)]
fn print_trace(trace: &ExecutionTrace<BaseElement>, _multiples_of: usize) {
    let trace_width = trace.width();
    let meta = TraceMetadata::from_trace_info(&trace.get_info());

    let mut state = vec![BaseElement::ZERO; trace_width];
    for i in 0..trace.length() {
        trace.read_row_into(i, &mut state);
        let state = TraceState::from_vec(meta.ctx_depth, meta.loop_depth, meta.stack_depth, &state);
        println!("{:?}", state);
    }
}