use super::{
    ByteReader, ByteWriter, Deserializable, Instruction, Node, ProcedureId, Serializable,
    SerializationError,
};
use crate::MAX_PUSH_INPUTS;
use num_enum::TryFromPrimitive;

mod deserialization;
mod serialization;

const IF_ELSE_OPCODE: u8 = 253;
const REPEAT_OPCODE: u8 = 254;
const WHILE_OPCODE: u8 = 255;

// OPERATION CODES ENUM
// ================================================================================================

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum OpCode {
    Assert = 0,
    AssertEq = 1,
    Assertz = 2,
    Add = 3,
    AddImm = 4,
    Sub = 5,
    SubImm = 6,
    Mul = 7,
    MulImm = 8,
    Div = 9,
    DivImm = 10,
    Neg = 11,
    Inv = 12,
    Incr = 13,
    Pow2 = 14,
    Exp = 15,
    ExpImm = 16,
    ExpBitLength = 17,
    Not = 18,
    And = 19,
    Or = 20,
    Xor = 21,
    Eq = 22,
    EqImm = 23,
    Neq = 24,
    NeqImm = 25,
    Eqw = 26,
    Lt = 27,
    Lte = 28,
    Gt = 29,
    Gte = 30,

    // ----- u32 manipulation ---------------------------------------------------------------
    U32Test = 31,
    U32TestW = 32,
    U32Assert = 33,
    U32Assert2 = 34,
    U32AssertW = 35,
    U32Split = 36,
    U32Cast = 37,
    U32CheckedAdd = 38,
    U32CheckedAddImm = 39,
    U32WrappingAdd = 40,
    U32WrappingAddImm = 41,
    U32OverflowingAdd = 42,
    U32OverflowingAddImm = 43,
    U32OverflowingAdd3 = 44,
    U32WrappingAdd3 = 45,
    U32CheckedSub = 46,
    U32CheckedSubImm = 47,
    U32WrappingSub = 48,
    U32WrappingSubImm = 49,
    U32OverflowingSub = 50,
    U32OverflowingSubImm = 51,
    U32CheckedMul = 52,
    U32CheckedMulImm = 53,
    U32WrappingMul = 54,
    U32WrappingMulImm = 55,
    U32OverflowingMul = 56,
    U32OverflowingMulImm = 57,
    U32OverflowingMadd = 58,
    U32WrappingMadd = 59,
    U32CheckedDiv = 60,
    U32CheckedDivImm = 61,
    U32UncheckedDiv = 62,
    U32UncheckedDivImm = 63,
    U32CheckedMod = 64,
    U32CheckedModImm = 65,
    U32UncheckedMod = 66,
    U32UncheckedModImm = 67,
    U32CheckedDivMod = 68,
    U32CheckedDivModImm = 69,
    U32UncheckedDivMod = 70,
    U32UncheckedDivModImm = 71,
    U32CheckedAnd = 72,
    U32CheckedOr = 73,
    U32CheckedXor = 74,
    U32CheckedNot = 75,
    U32CheckedShr = 76,
    U32CheckedShrImm = 77,
    U32UncheckedShr = 78,
    U32UncheckedShrImm = 79,
    U32CheckedShl = 80,
    U32CheckedShlImm = 81,
    U32UncheckedShl = 82,
    U32UncheckedShlImm = 83,
    U32CheckedRotr = 84,
    U32CheckedRotrImm = 85,
    U32UncheckedRotr = 86,
    U32UncheckedRotrImm = 87,
    U32CheckedRotl = 88,
    U32CheckedRotlImm = 89,
    U32UncheckedRotl = 90,
    U32UncheckedRotlImm = 91,
    U32CheckedEq = 92,
    U32CheckedEqImm = 93,
    U32CheckedNeq = 94,
    U32CheckedNeqImm = 95,
    U32CheckedLt = 96,
    U32UncheckedLt = 97,
    U32CheckedLte = 98,
    U32UncheckedLte = 99,
    U32CheckedGt = 100,
    U32UncheckedGt = 101,
    U32CheckedGte = 102,
    U32UncheckedGte = 103,
    U32CheckedMin = 104,
    U32UncheckedMin = 105,
    U32CheckedMax = 106,
    U32UncheckedMax = 107,

    // ----- stack manipulation ---------------------------------------------------------------
    Drop = 108,
    DropW = 109,
    PadW = 110,
    Dup0 = 111,
    Dup1 = 112,
    Dup2 = 113,
    Dup3 = 114,
    Dup4 = 115,
    Dup5 = 116,
    Dup6 = 117,
    Dup7 = 118,
    Dup8 = 119,
    Dup9 = 120,
    Dup10 = 121,
    Dup11 = 122,
    Dup12 = 123,
    Dup13 = 124,
    Dup14 = 125,
    Dup15 = 126,
    DupW0 = 127,
    DupW1 = 128,
    DupW2 = 129,
    DupW3 = 130,
    Swap1 = 131,
    Swap2 = 132,
    Swap3 = 133,
    Swap4 = 134,
    Swap5 = 135,
    Swap6 = 136,
    Swap7 = 137,
    Swap8 = 138,
    Swap9 = 139,
    Swap10 = 140,
    Swap11 = 141,
    Swap12 = 142,
    Swap13 = 143,
    Swap14 = 144,
    Swap15 = 145,
    SwapW1 = 146,
    SwapW2 = 147,
    SwapW3 = 148,
    SwapDW = 149,
    MovUp2 = 150,
    MovUp3 = 151,
    MovUp4 = 152,
    MovUp5 = 153,
    MovUp6 = 154,
    MovUp7 = 155,
    MovUp8 = 156,
    MovUp9 = 157,
    MovUp10 = 158,
    MovUp11 = 159,
    MovUp12 = 160,
    MovUp13 = 161,
    MovUp14 = 162,
    MovUp15 = 163,
    MovUpW2 = 164,
    MovUpW3 = 165,
    MovDn2 = 166,
    MovDn3 = 167,
    MovDn4 = 168,
    MovDn5 = 169,
    MovDn6 = 170,
    MovDn7 = 171,
    MovDn8 = 172,
    MovDn9 = 173,
    MovDn10 = 174,
    MovDn11 = 175,
    MovDn12 = 176,
    MovDn13 = 177,
    MovDn14 = 178,
    MovDn15 = 179,
    MovDnW2 = 180,
    MovDnW3 = 181,
    CSwap = 182,
    CSwapW = 183,
    CDrop = 184,
    CDropW = 185,

    // ----- input / output operations --------------------------------------------------------
    PushU8 = 186,
    PushU16 = 187,
    PushU32 = 188,
    PushFelt = 189,
    PushWord = 190,
    PushU8List = 191,
    PushU16List = 192,
    PushU32List = 193,
    PushFeltList = 194,

    Locaddr = 195,
    Sdepth = 196,
    Caller = 197,

    MemLoad = 198,
    MemLoadImm = 199,
    MemLoadW = 200,
    MemLoadWImm = 201,
    LocLoad = 202,
    LocLoadW = 203,
    MemStore = 204,
    MemStoreImm = 205,
    LocStore = 206,
    MemStoreW = 207,
    MemStoreWImm = 208,
    LocStoreW = 209,

    MemStream = 210,
    AdvPipe = 211,

    AdvPush = 212,
    AdvLoadW = 213,

    AdvU64Div = 214,
    AdvKeyval = 215,
    AdvMem = 216,

    // ----- cryptographic operations ---------------------------------------------------------
    RPHash = 217,
    RPPerm = 218,
    MTreeGet = 219,
    MTreeSet = 220,
    MTreeCwm = 221,

    // ----- exec / call ----------------------------------------------------------------------
    ExecLocal = 222,
    ExecImported = 223,
    CallLocal = 224,
    CallImported = 225,
    SysCall = 226,
}

impl Serializable for OpCode {
    fn write_into(&self, target: &mut ByteWriter) -> Result<(), SerializationError> {
        target.write_u8(*self as u8);
        Ok(())
    }
}

impl Deserializable for OpCode {
    fn read_from(bytes: &mut ByteReader) -> Result<Self, SerializationError> {
        let value = bytes.read_u8()?;
        Self::try_from(value).map_err(|_| SerializationError::InvalidOpCode)
    }
}
