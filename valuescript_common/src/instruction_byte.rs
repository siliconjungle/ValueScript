#[derive(Debug, Clone, PartialEq)]
pub enum InstructionByte {
  End = 0x00,
  Mov = 0x01,
  OpInc = 0x02,
  OpDec = 0x03,
  OpPlus = 0x04,
  OpMinus = 0x05,
  OpMul = 0x06,
  OpDiv = 0x07,
  OpMod = 0x08,
  OpExp = 0x09,
  OpEq = 0x0a,
  OpNe = 0x0b,
  OpTripleEq = 0x0c,
  OpTripleNe = 0x0d,
  OpAnd = 0x0e,
  OpOr = 0x0f,
  OpNot = 0x10,
  OpLess = 0x11,
  OpLessEq = 0x12,
  OpGreater = 0x13,
  OpGreaterEq = 0x14,
  OpNullishCoalesce = 0x15,
  OpOptionalChain = 0x16,
  OpBitAnd = 0x17,
  OpBitOr = 0x18,
  OpBitNot = 0x19,
  OpBitXor = 0x1a,
  OpLeftShift = 0x1b,
  OpRightShift = 0x1c,
  OpRightShiftUnsigned = 0x1d,
  TypeOf = 0x1e,
  InstanceOf = 0x1f,
  In = 0x20,
  Call = 0x21,
  Apply = 0x22,
  ConstApply = 0x23,
  Bind = 0x24,
  Sub = 0x25,
  SubMov = 0x26,
  SubCall = 0x27,
  Jmp = 0x28,
  JmpIf = 0x29,
  JmpIfNot = 0x2a,
  UnaryPlus = 0x2b,
  UnaryMinus = 0x2c,
  New = 0x2d,
  Throw = 0x2e,
  Import = 0x2f,
  ImportStar = 0x30,
  SetCatch = 0x31,
  UnsetCatch = 0x32,
  ConstSubCall = 0x33,
  RequireMutableThis = 0x34,
  ThisSubCall = 0x35,
  Next = 0x36,
  UnpackIterRes = 0x37,
  Cat = 0x38,
  Yield = 0x39,
  YieldStar = 0x3a,
}

impl InstructionByte {
  pub fn from_byte(byte: u8) -> InstructionByte {
    use InstructionByte::*;

    match byte {
      0x00 => End,
      0x01 => Mov,
      0x02 => OpInc,
      0x03 => OpDec,
      0x04 => OpPlus,
      0x05 => OpMinus,
      0x06 => OpMul,
      0x07 => OpDiv,
      0x08 => OpMod,
      0x09 => OpExp,
      0x0a => OpEq,
      0x0b => OpNe,
      0x0c => OpTripleEq,
      0x0d => OpTripleNe,
      0x0e => OpAnd,
      0x0f => OpOr,
      0x10 => OpNot,
      0x11 => OpLess,
      0x12 => OpLessEq,
      0x13 => OpGreater,
      0x14 => OpGreaterEq,
      0x15 => OpNullishCoalesce,
      0x16 => OpOptionalChain,
      0x17 => OpBitAnd,
      0x18 => OpBitOr,
      0x19 => OpBitNot,
      0x1a => OpBitXor,
      0x1b => OpLeftShift,
      0x1c => OpRightShift,
      0x1d => OpRightShiftUnsigned,
      0x1e => TypeOf,
      0x1f => InstanceOf,
      0x20 => In,
      0x21 => Call,
      0x22 => Apply,
      0x23 => ConstApply,
      0x24 => Bind,
      0x25 => Sub,
      0x26 => SubMov,
      0x27 => SubCall,
      0x28 => Jmp,
      0x29 => JmpIf,
      0x2a => JmpIfNot,
      0x2b => UnaryPlus,
      0x2c => UnaryMinus,
      0x2d => New,
      0x2e => Throw,
      0x2f => Import,
      0x30 => ImportStar,
      0x31 => SetCatch,
      0x32 => UnsetCatch,
      0x33 => ConstSubCall,
      0x34 => RequireMutableThis,
      0x35 => ThisSubCall,
      0x36 => Next,
      0x37 => UnpackIterRes,
      0x38 => Cat,
      0x39 => Yield,
      0x3a => YieldStar,

      _ => panic!("Unrecognized instruction: {}", byte),
    }
  }
}
