use vm_core::utils::collections::Vec;
use vm_core::Felt;

use crate::ProcedureId;

// Nodes
// ================================================================================================

/// A node in a AST that can represent a block, instruction or a control flow.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Node {
    Instruction(Instruction),
    IfElse(Vec<Node>, Vec<Node>),
    Repeat(usize, Vec<Node>),
    While(Vec<Node>),
}

/// This holds the list of instructions supported in a Miden program.
/// This instruction list is used to hold reference to the instruction, and future be
/// used for MAST generation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Instruction {
    Assert,
    AssertEq,
    Assertz,
    Add,
    AddImm(Felt),
    Sub,
    SubImm(Felt),
    Mul,
    MulImm(Felt),
    Div,
    DivImm(Felt),
    Neg,
    Inv,
    Pow2,
    Exp,
    ExpImm(Felt),
    ExpBitLength(u8),
    Not,
    And,
    Or,
    Xor,
    Eq,
    EqImm(Felt),
    Neq,
    NeqImm(Felt),
    Eqw,
    Lt,
    Lte,
    Gt,
    Gte,

    // ----- u32 manipulation ---------------------------------------------------------------
    U32Test,
    U32TestW,
    U32Assert,
    U32Assert2,
    U32AssertW,
    U32Split,
    U32Cast,
    U32CheckedAdd,
    U32CheckedAddImm(u32),
    U32WrappingAdd,
    U32WrappingAddImm(u32),
    U32OverflowingAdd,
    U32OverflowingAddImm(u32),
    U32OverflowingAdd3,
    U32WrappingAdd3,
    U32CheckedSub,
    U32CheckedSubImm(u32),
    U32WrappingSub,
    U32WrappingSubImm(u32),
    U32OverflowingSub,
    U32OverflowingSubImm(u32),
    U32CheckedMul,
    U32CheckedMulImm(u32),
    U32WrappingMul,
    U32WrappingMulImm(u32),
    U32OverflowingMul,
    U32OverflowingMulImm(u32),
    U32OverflowingMadd,
    U32WrappingMadd,
    U32CheckedDiv,
    U32CheckedDivImm(u32),
    U32UncheckedDiv,
    U32UncheckedDivImm(u32),
    U32CheckedMod,
    U32CheckedModImm(u32),
    U32UncheckedMod,
    U32UncheckedModImm(u32),
    U32CheckedDivMod,
    U32CheckedDivModImm(u32),
    U32UncheckedDivMod,
    U32UncheckedDivModImm(u32),
    U32CheckedAnd,
    U32CheckedOr,
    U32CheckedXor,
    U32CheckedNot,
    U32CheckedShr,
    U32CheckedShrImm(u8),
    U32UncheckedShr,
    U32UncheckedShrImm(u8),
    U32CheckedShl,
    U32CheckedShlImm(u8),
    U32UncheckedShl,
    U32UncheckedShlImm(u8),
    U32CheckedRotr,
    U32CheckedRotrImm(u8),
    U32UncheckedRotr,
    U32UncheckedRotrImm(u8),
    U32CheckedRotl,
    U32CheckedRotlImm(u8),
    U32UncheckedRotl,
    U32UncheckedRotlImm(u8),
    U32CheckedEq,
    U32CheckedEqImm(u32),
    U32CheckedNeq,
    U32CheckedNeqImm(u32),
    U32CheckedLt,
    U32UncheckedLt,
    U32CheckedLte,
    U32UncheckedLte,
    U32CheckedGt,
    U32UncheckedGt,
    U32CheckedGte,
    U32UncheckedGte,
    U32CheckedMin,
    U32UncheckedMin,
    U32CheckedMax,
    U32UncheckedMax,

    // ----- stack manipulation ---------------------------------------------------------------
    Drop,
    DropW,
    PadW,
    Dup0,
    Dup1,
    Dup2,
    Dup3,
    Dup4,
    Dup5,
    Dup6,
    Dup7,
    Dup8,
    Dup9,
    Dup10,
    Dup11,
    Dup12,
    Dup13,
    Dup14,
    Dup15,
    DupW0,
    DupW1,
    DupW2,
    DupW3,
    Swap,
    Swap2,
    Swap3,
    Swap4,
    Swap5,
    Swap6,
    Swap7,
    Swap8,
    Swap9,
    Swap10,
    Swap11,
    Swap12,
    Swap13,
    Swap14,
    Swap15,
    SwapW,
    SwapW2,
    SwapW3,
    SwapDW,
    MovUp2,
    MovUp3,
    MovUp4,
    MovUp5,
    MovUp6,
    MovUp7,
    MovUp8,
    MovUp9,
    MovUp10,
    MovUp11,
    MovUp12,
    MovUp13,
    MovUp14,
    MovUp15,
    MovUpW2,
    MovUpW3,
    MovDn2,
    MovDn3,
    MovDn4,
    MovDn5,
    MovDn6,
    MovDn7,
    MovDn8,
    MovDn9,
    MovDn10,
    MovDn11,
    MovDn12,
    MovDn13,
    MovDn14,
    MovDn15,
    MovDnW2,
    MovDnW3,
    CSwap,
    CSwapW,
    CDrop,
    CDropW,

    // ----- input / output operations --------------------------------------------------------
    PushConstants(Vec<Felt>),
    Locaddr(Felt),
    Sdepth,
    Caller,

    MemLoad,
    MemLoadImm(Felt),
    MemLoadW,
    MemLoadWImm(Felt),
    LocLoad(Felt),
    LocLoadW(Felt),

    MemStore,
    MemStoreImm(Felt),
    LocStore(Felt),
    MemStoreW,
    MemStoreWImm(Felt),
    LocStoreW(Felt),

    MemStream,
    AdvPipe,

    AdvPush(u8),
    AdvLoadW,

    AdvU64Div,
    AdvKeyval,
    AdvMem(u32, u32),

    // ----- cryptographic operations ---------------------------------------------------------
    RPHash,
    RPPerm,
    MTreeGet,
    MTreeSet,
    MTreeCwm,

    // ----- exec / call ----------------------------------------------------------------------
    ExecLocal(u16),
    ExecImported(ProcedureId),
    CallLocal(u16),
    CallImported(ProcedureId),
    SysCall(ProcedureId),
}
