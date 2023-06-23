use std::hash::{Hash as HashTrait, Hasher};

use num_bigint::BigInt;
use valuescript_common::InstructionByte;

use crate::{assembler::ValueType, expression_compiler::CompiledExpression};

#[derive(Debug, Clone)]
pub struct Module {
  pub export_default: Value,
  pub export_star: Object,
  pub definitions: Vec<Definition>,
}

impl Module {
  pub fn as_lines(&self) -> Vec<String> {
    let assembly_str = self.to_string();
    let assembly_lines = assembly_str.split("\n");
    let assembly_lines_vec = assembly_lines.map(|s| s.to_string()).collect();

    return assembly_lines_vec;
  }
}

impl Default for Module {
  fn default() -> Self {
    Module {
      export_default: Value::Void,
      export_star: Object::default(),
      definitions: Vec::default(),
    }
  }
}

impl std::fmt::Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.export_star.properties.len() == 0 {
      write!(f, "export {} {}", self.export_default, self.export_star)?;
    } else {
      write!(f, "export {} {{\n", self.export_default)?;

      for (name, value) in &self.export_star.properties {
        write!(f, "  {}: {},\n", name, value)?;
      }

      write!(f, "}}")?;
    }

    for definition in &self.definitions {
      write!(f, "\n\n{}", definition)?;
    }

    return Ok(());
  }
}

#[derive(Debug, Clone)]
pub struct Definition {
  pub pointer: Pointer,
  pub content: DefinitionContent,
}

impl std::fmt::Display for Definition {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{} = {}", self.pointer, self.content)
  }
}

#[derive(Debug, Clone)]
pub enum DefinitionContent {
  Function(Function),
  Class(Class),
  Value(Value),
  Lazy(Lazy),
}

impl std::fmt::Display for DefinitionContent {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      DefinitionContent::Function(function) => {
        write!(f, "{}", function)
      }
      DefinitionContent::Class(class) => {
        write!(f, "{}", class)
      }
      DefinitionContent::Value(value) => {
        write!(f, "{}", value)
      }
      DefinitionContent::Lazy(lazy) => {
        write!(f, "{}", lazy)
      }
    }
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Pointer {
  pub name: String,
}

impl std::fmt::Display for Pointer {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "@{}", self.name)
  }
}

#[derive(Default, Debug, Clone)]
pub struct Function {
  pub is_generator: bool,
  pub parameters: Vec<Register>,
  pub body: Vec<FnLine>,
}

impl std::fmt::Display for Function {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self.is_generator {
      false => write!(f, "function(")?,
      true => write!(f, "function*(")?,
    }

    for (i, parameter) in self.parameters.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}", parameter)?;
    }
    write!(f, ") {{\n")?;
    for fn_line in &self.body {
      match fn_line {
        FnLine::Instruction(instruction) => write!(f, "  {}\n", instruction)?,
        FnLine::Label(label) => write!(f, "{}\n", label)?,
        FnLine::Empty => write!(f, "\n")?,
        FnLine::Comment(message) => write!(f, "  // {}\n", message)?,
      }
    }
    write!(f, "}}")
  }
}

#[derive(Debug, Clone)]
pub struct Class {
  pub constructor: Value,
  pub prototype: Value,
  pub static_: Value,
}

impl std::fmt::Display for Class {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    writeln!(f, "class {{")?;

    writeln!(f, "  constructor: {},", self.constructor)?;

    write!(f, "  prototype: ")?;

    match &self.prototype {
      Value::Object(object) => {
        if object.properties.len() == 0 {
          writeln!(f, "{{}},")?;
        } else {
          write!(f, "{{\n")?;
          for (name, method) in &object.properties {
            write!(f, "    {}: {},\n", name, method)?;
          }
          writeln!(f, "  }},")?;
        }
      }
      _ => {
        writeln!(f, "{},", self.prototype)?;
      }
    }

    write!(f, "  static: ")?;

    match &self.static_ {
      Value::Object(object) => {
        if object.properties.len() == 0 {
          writeln!(f, "{{}},")?;
        } else {
          write!(f, "{{\n")?;
          for (name, method) in &object.properties {
            write!(f, "    {}: {},\n", name, method)?;
          }
          writeln!(f, "  }},")?;
        }
      }
      _ => {
        writeln!(f, "{},", self.prototype)?;
      }
    }

    write!(f, "}}")?;

    return Ok(());
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Register {
  pub take: bool,
  pub name: String,
}

impl Register {
  pub fn return_() -> Self {
    Register {
      take: false,
      name: "return".to_string(),
    }
  }

  pub fn this() -> Self {
    Register {
      take: false,
      name: "this".to_string(),
    }
  }

  pub fn named(name: String) -> Self {
    Register { take: false, name }
  }

  pub fn ignore() -> Self {
    Register {
      take: false,
      name: "ignore".to_string(),
    }
  }

  pub fn take(&self) -> Self {
    Register {
      take: true,
      name: self.name.clone(),
    }
  }

  pub fn is_return(&self) -> bool {
    return self.name == "return";
  }

  pub fn is_this(&self) -> bool {
    return self.name == "this";
  }

  pub fn is_named(&self) -> bool {
    match self.name.as_str() {
      "return" | "this" | "ignore" => false,
      _ => true,
    }
  }

  pub fn is_ignore(&self) -> bool {
    return self.name == "ignore";
  }

  pub fn value_type(&self) -> ValueType {
    if self.take {
      ValueType::TakeRegister
    } else {
      ValueType::Register
    }
  }
}

impl std::fmt::Display for Register {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "%")?;

    if self.take {
      write!(f, "!")?;
    }

    write!(f, "{}", self.name)
  }
}

#[derive(Debug, Clone)]
pub enum FnLine {
  Instruction(Instruction),
  Label(Label),
  Empty,
  Comment(String),
}

impl std::fmt::Display for FnLine {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      FnLine::Instruction(instruction) => {
        write!(f, "{}", instruction)
      }
      FnLine::Label(label) => {
        write!(f, "{}", label)
      }
      FnLine::Empty => Ok(()),
      FnLine::Comment(message) => {
        write!(f, "// {}", message)
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Label {
  pub name: String,
}

impl Label {
  pub fn ref_(&self) -> LabelRef {
    LabelRef {
      name: self.name.clone(),
    }
  }
}

impl std::fmt::Display for Label {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}:", self.name)
  }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct LabelRef {
  pub name: String,
}

impl std::fmt::Display for LabelRef {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, ":{}", self.name)
  }
}

#[derive(Debug, Clone)]
pub enum Instruction {
  End,
  Mov(Value, Register),
  OpInc(Register),
  OpDec(Register),
  OpPlus(Value, Value, Register),
  OpMinus(Value, Value, Register),
  OpMul(Value, Value, Register),
  OpDiv(Value, Value, Register),
  OpMod(Value, Value, Register),
  OpExp(Value, Value, Register),
  OpEq(Value, Value, Register),
  OpNe(Value, Value, Register),
  OpTripleEq(Value, Value, Register),
  OpTripleNe(Value, Value, Register),
  OpAnd(Value, Value, Register),
  OpOr(Value, Value, Register),
  OpNot(Value, Register),
  OpLess(Value, Value, Register),
  OpLessEq(Value, Value, Register),
  OpGreater(Value, Value, Register),
  OpGreaterEq(Value, Value, Register),
  OpNullishCoalesce(Value, Value, Register),
  OpOptionalChain(Value, Value, Register),
  OpBitAnd(Value, Value, Register),
  OpBitOr(Value, Value, Register),
  OpBitNot(Value, Register),
  OpBitXor(Value, Value, Register),
  OpLeftShift(Value, Value, Register),
  OpRightShift(Value, Value, Register),
  OpRightShiftUnsigned(Value, Value, Register),
  TypeOf(Value, Register),
  InstanceOf(Value, Value, Register),
  In(Value, Value, Register),
  Call(Value, Value, Register),
  Apply(Value, Value, Value, Register),
  Bind(Value, Value, Register),
  Sub(Value, Value, Register),
  SubMov(Value, Value, Register),
  SubCall(Value, Value, Value, Register),
  Jmp(LabelRef),
  JmpIf(Value, LabelRef),
  UnaryPlus(Value, Register),
  UnaryMinus(Value, Register),
  New(Value, Value, Register),
  Throw(Value),
  Import(Value, Register),
  ImportStar(Value, Register),
  SetCatch(LabelRef, Register),
  UnsetCatch,
  ConstSubCall(Value, Value, Value, Register),
  RequireMutableThis,
  ThisSubCall(Value, Value, Value, Register),
  Next(Register, Register),
  UnpackIterRes(Register, Register, Register),
  Cat(Value, Register),
  Yield(Value, Register),
  YieldStar(Value, Register),
}

pub enum InstructionFieldMut<'a> {
  Value(&'a mut Value),
  Register(&'a mut Register),
  LabelRef(&'a mut LabelRef),
}

impl Instruction {
  pub fn visit_fields_mut<F>(&mut self, visit: &mut F)
  where
    F: FnMut(InstructionFieldMut) -> (),
  {
    use Instruction::*;

    match self {
      End => {}
      Mov(arg, dst)
      | OpNot(arg, dst)
      | OpBitNot(arg, dst)
      | TypeOf(arg, dst)
      | UnaryPlus(arg, dst)
      | UnaryMinus(arg, dst)
      | Import(arg, dst)
      | ImportStar(arg, dst)
      | Cat(arg, dst)
      | Yield(arg, dst)
      | YieldStar(arg, dst) => {
        visit(InstructionFieldMut::Value(arg));
        visit(InstructionFieldMut::Register(dst));
      }

      OpInc(arg) | OpDec(arg) => {
        visit(InstructionFieldMut::Register(arg));
      }

      OpPlus(left, right, dst)
      | OpMinus(left, right, dst)
      | OpMul(left, right, dst)
      | OpDiv(left, right, dst)
      | OpMod(left, right, dst)
      | OpExp(left, right, dst)
      | OpEq(left, right, dst)
      | OpNe(left, right, dst)
      | OpTripleEq(left, right, dst)
      | OpTripleNe(left, right, dst)
      | OpAnd(left, right, dst)
      | OpOr(left, right, dst)
      | OpLess(left, right, dst)
      | OpLessEq(left, right, dst)
      | OpGreater(left, right, dst)
      | OpGreaterEq(left, right, dst)
      | OpNullishCoalesce(left, right, dst)
      | OpOptionalChain(left, right, dst)
      | OpBitAnd(left, right, dst)
      | OpBitOr(left, right, dst)
      | OpBitXor(left, right, dst)
      | OpLeftShift(left, right, dst)
      | OpRightShift(left, right, dst)
      | OpRightShiftUnsigned(left, right, dst)
      | InstanceOf(left, right, dst)
      | In(left, right, dst)
      | Call(left, right, dst)
      | Bind(left, right, dst)
      | Sub(left, right, dst)
      | SubMov(left, right, dst)
      | New(left, right, dst) => {
        visit(InstructionFieldMut::Value(left));
        visit(InstructionFieldMut::Value(right));
        visit(InstructionFieldMut::Register(dst));
      }

      Apply(a1, a2, a3, dst)
      | SubCall(a1, a2, a3, dst)
      | ConstSubCall(a1, a2, a3, dst)
      | ThisSubCall(a1, a2, a3, dst) => {
        visit(InstructionFieldMut::Value(a1));
        visit(InstructionFieldMut::Value(a2));
        visit(InstructionFieldMut::Value(a3));
        visit(InstructionFieldMut::Register(dst));
      }

      Jmp(label_ref) => {
        visit(InstructionFieldMut::LabelRef(label_ref));
      }

      JmpIf(cond, label_ref) => {
        visit(InstructionFieldMut::Value(cond));
        visit(InstructionFieldMut::LabelRef(label_ref));
      }

      Throw(ex) => {
        visit(InstructionFieldMut::Value(ex));
      }

      SetCatch(label_ref, dst) => {
        visit(InstructionFieldMut::LabelRef(label_ref));
        visit(InstructionFieldMut::Register(dst));
      }

      Next(iterable, dst) => {
        visit(InstructionFieldMut::Register(iterable));
        visit(InstructionFieldMut::Register(dst));
      }

      UnpackIterRes(iter_res, value_dst, done_dst) => {
        visit(InstructionFieldMut::Register(iter_res));
        visit(InstructionFieldMut::Register(value_dst));
        visit(InstructionFieldMut::Register(done_dst));
      }

      UnsetCatch | RequireMutableThis => {}
    }
  }
}

impl std::fmt::Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Instruction::End => write!(f, "end"),
      Instruction::Mov(value, register) => {
        write!(f, "mov {} {}", value, register)
      }
      Instruction::OpInc(register) => write!(f, "op++ {}", register),
      Instruction::OpDec(register) => write!(f, "op-- {}", register),
      Instruction::OpPlus(lhs, rhs, register) => {
        write!(f, "op+ {} {} {}", lhs, rhs, register)
      }
      Instruction::OpMinus(lhs, rhs, register) => {
        write!(f, "op- {} {} {}", lhs, rhs, register)
      }
      Instruction::OpMul(lhs, rhs, register) => {
        write!(f, "op* {} {} {}", lhs, rhs, register)
      }
      Instruction::OpDiv(lhs, rhs, register) => {
        write!(f, "op/ {} {} {}", lhs, rhs, register)
      }
      Instruction::OpMod(lhs, rhs, register) => {
        write!(f, "op% {} {} {}", lhs, rhs, register)
      }
      Instruction::OpExp(lhs, rhs, register) => {
        write!(f, "op** {} {} {}", lhs, rhs, register)
      }
      Instruction::OpEq(lhs, rhs, register) => {
        write!(f, "op== {} {} {}", lhs, rhs, register)
      }
      Instruction::OpNe(lhs, rhs, register) => {
        write!(f, "op!= {} {} {}", lhs, rhs, register)
      }
      Instruction::OpTripleEq(lhs, rhs, register) => {
        write!(f, "op=== {} {} {}", lhs, rhs, register)
      }
      Instruction::OpTripleNe(lhs, rhs, register) => {
        write!(f, "op!== {} {} {}", lhs, rhs, register)
      }
      Instruction::OpAnd(lhs, rhs, register) => {
        write!(f, "op&& {} {} {}", lhs, rhs, register)
      }
      Instruction::OpOr(lhs, rhs, register) => {
        write!(f, "op|| {} {} {}", lhs, rhs, register)
      }
      Instruction::OpNot(value, register) => {
        write!(f, "op! {} {}", value, register)
      }
      Instruction::OpLess(lhs, rhs, register) => {
        write!(f, "op< {} {} {}", lhs, rhs, register)
      }
      Instruction::OpLessEq(lhs, rhs, register) => {
        write!(f, "op<= {} {} {}", lhs, rhs, register)
      }
      Instruction::OpGreater(lhs, rhs, register) => {
        write!(f, "op> {} {} {}", lhs, rhs, register)
      }
      Instruction::OpGreaterEq(lhs, rhs, register) => {
        write!(f, "op>= {} {} {}", lhs, rhs, register)
      }
      Instruction::OpNullishCoalesce(lhs, rhs, register) => {
        write!(f, "op?? {} {} {}", lhs, rhs, register)
      }
      Instruction::OpOptionalChain(lhs, rhs, register) => {
        write!(f, "op?. {} {} {}", lhs, rhs, register)
      }
      Instruction::OpBitAnd(lhs, rhs, register) => {
        write!(f, "op& {} {} {}", lhs, rhs, register)
      }
      Instruction::OpBitOr(lhs, rhs, register) => {
        write!(f, "op| {} {} {}", lhs, rhs, register)
      }
      Instruction::OpBitNot(value, register) => {
        write!(f, "op~ {} {}", value, register)
      }
      Instruction::OpBitXor(lhs, rhs, register) => {
        write!(f, "op^ {} {} {}", lhs, rhs, register)
      }
      Instruction::OpLeftShift(lhs, rhs, register) => {
        write!(f, "op<< {} {} {}", lhs, rhs, register)
      }
      Instruction::OpRightShift(lhs, rhs, register) => {
        write!(f, "op>> {} {} {}", lhs, rhs, register)
      }
      Instruction::OpRightShiftUnsigned(lhs, rhs, register) => {
        write!(f, "op>>> {} {} {}", lhs, rhs, register)
      }
      Instruction::TypeOf(value, register) => {
        write!(f, "typeof {} {}", value, register)
      }
      Instruction::InstanceOf(lhs, rhs, register) => {
        write!(f, "instanceof {} {} {}", lhs, rhs, register)
      }
      Instruction::In(lhs, rhs, register) => {
        write!(f, "in {} {} {}", lhs, rhs, register)
      }
      Instruction::Call(value, args, register) => {
        write!(f, "call {} {} {}", value, args, register)
      }
      Instruction::Apply(value, this, args, register) => {
        write!(f, "apply {} {} {} {}", value, this, args, register)
      }
      Instruction::Bind(value, args, register) => {
        write!(f, "bind {} {} {}", value, args, register)
      }
      Instruction::Sub(lhs, rhs, register) => {
        write!(f, "sub {} {} {}", lhs, rhs, register)
      }
      Instruction::SubMov(subscript, value, register) => {
        write!(f, "submov {} {} {}", subscript, value, register)
      }
      Instruction::SubCall(obj, subscript, args, register) => {
        write!(f, "subcall {} {} {} {}", obj, subscript, args, register)
      }
      Instruction::Jmp(label_ref) => write!(f, "jmp {}", label_ref),
      Instruction::JmpIf(value, label_ref) => {
        write!(f, "jmpif {} {}", value, label_ref)
      }
      Instruction::UnaryPlus(value, register) => {
        write!(f, "unary+ {} {}", value, register)
      }
      Instruction::UnaryMinus(value, register) => {
        write!(f, "unary- {} {}", value, register)
      }
      Instruction::New(value, args, register) => {
        write!(f, "new {} {} {}", value, args, register)
      }
      Instruction::Throw(value) => write!(f, "throw {}", value),
      Instruction::Import(value, register) => {
        write!(f, "import {} {}", value, register)
      }
      Instruction::ImportStar(value, register) => {
        write!(f, "import* {} {}", value, register)
      }
      Instruction::SetCatch(label, register) => {
        write!(f, "set_catch {} {}", label, register)
      }
      Instruction::UnsetCatch => write!(f, "unset_catch"),
      Instruction::ConstSubCall(obj, subscript, args, register) => {
        write!(
          f,
          "const_subcall {} {} {} {}",
          obj, subscript, args, register
        )
      }
      Instruction::RequireMutableThis => write!(f, "require_mutable_this"),
      Instruction::ThisSubCall(obj, subscript, args, register) => {
        write!(
          f,
          "this_subcall {} {} {} {}",
          obj, subscript, args, register
        )
      }
      Instruction::Next(obj, register) => {
        write!(f, "next {} {}", obj, register)
      }
      Instruction::UnpackIterRes(obj, value_register, done_register) => {
        write!(
          f,
          "unpack_iter_res {} {} {}",
          obj, value_register, done_register
        )
      }
      Instruction::Cat(iterables, register) => {
        write!(f, "cat {} {}", iterables, register)
      }
      Instruction::Yield(value, register) => {
        write!(f, "yield {} {}", value, register)
      }
      Instruction::YieldStar(value, register) => {
        write!(f, "yield* {} {}", value, register)
      }
    }
  }
}

impl Instruction {
  pub fn byte(&self) -> InstructionByte {
    use Instruction::*;

    // TODO: Define this in one place only
    match self {
      End => InstructionByte::End,
      Mov(..) => InstructionByte::Mov,
      OpInc(..) => InstructionByte::OpInc,
      OpDec(..) => InstructionByte::OpDec,
      OpPlus(..) => InstructionByte::OpPlus,
      OpMinus(..) => InstructionByte::OpMinus,
      OpMul(..) => InstructionByte::OpMul,
      OpDiv(..) => InstructionByte::OpDiv,
      OpMod(..) => InstructionByte::OpMod,
      OpExp(..) => InstructionByte::OpExp,
      OpEq(..) => InstructionByte::OpEq,
      OpNe(..) => InstructionByte::OpNe,
      OpTripleEq(..) => InstructionByte::OpTripleEq,
      OpTripleNe(..) => InstructionByte::OpTripleNe,
      OpAnd(..) => InstructionByte::OpAnd,
      OpOr(..) => InstructionByte::OpOr,
      OpNot(..) => InstructionByte::OpNot,
      OpLess(..) => InstructionByte::OpLess,
      OpLessEq(..) => InstructionByte::OpLessEq,
      OpGreater(..) => InstructionByte::OpGreater,
      OpGreaterEq(..) => InstructionByte::OpGreaterEq,
      OpNullishCoalesce(..) => InstructionByte::OpNullishCoalesce,
      OpOptionalChain(..) => InstructionByte::OpOptionalChain,
      OpBitAnd(..) => InstructionByte::OpBitAnd,
      OpBitOr(..) => InstructionByte::OpBitOr,
      OpBitNot(..) => InstructionByte::OpBitNot,
      OpBitXor(..) => InstructionByte::OpBitXor,
      OpLeftShift(..) => InstructionByte::OpLeftShift,
      OpRightShift(..) => InstructionByte::OpRightShift,
      OpRightShiftUnsigned(..) => InstructionByte::OpRightShiftUnsigned,
      TypeOf(..) => InstructionByte::TypeOf,
      InstanceOf(..) => InstructionByte::InstanceOf,
      In(..) => InstructionByte::In,
      Call(..) => InstructionByte::Call,
      Apply(..) => InstructionByte::Apply,
      Bind(..) => InstructionByte::Bind,
      Sub(..) => InstructionByte::Sub,
      SubMov(..) => InstructionByte::SubMov,
      SubCall(..) => InstructionByte::SubCall,
      Jmp(..) => InstructionByte::Jmp,
      JmpIf(..) => InstructionByte::JmpIf,
      UnaryPlus(..) => InstructionByte::UnaryPlus,
      UnaryMinus(..) => InstructionByte::UnaryMinus,
      New(..) => InstructionByte::New,
      Throw(..) => InstructionByte::Throw,
      Import(..) => InstructionByte::Import,
      ImportStar(..) => InstructionByte::ImportStar,
      SetCatch(..) => InstructionByte::SetCatch,
      UnsetCatch => InstructionByte::UnsetCatch,
      ConstSubCall(..) => InstructionByte::ConstSubCall,
      RequireMutableThis => InstructionByte::RequireMutableThis,
      ThisSubCall(..) => InstructionByte::ThisSubCall,
      Next(..) => InstructionByte::Next,
      UnpackIterRes(..) => InstructionByte::UnpackIterRes,
      Cat(..) => InstructionByte::Cat,
      Yield(..) => InstructionByte::Yield,
      YieldStar(..) => InstructionByte::YieldStar,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(Number),
  BigInt(BigInt),
  String(String),
  Array(Box<Array>),
  Object(Box<Object>),
  Register(Register),
  Pointer(Pointer),
  Builtin(Builtin),
}

impl Default for Value {
  fn default() -> Self {
    Value::Void
  }
}

#[derive(Debug, Clone)]
pub struct Number(pub f64);

impl PartialEq for Number {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for Number {
  fn assert_receiver_is_total_eq(&self) {}
}

impl HashTrait for Number {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.0.to_bits());
  }
}

impl Value {
  pub fn to_ce(self) -> CompiledExpression {
    CompiledExpression::new(self, vec![])
  }

  pub fn visit_values_mut<F>(&mut self, visit: &mut F)
  where
    F: FnMut(&mut Value) -> (),
  {
    visit(self);

    match self {
      Value::Array(array) => {
        for item in &mut array.values {
          item.visit_values_mut(visit);
        }
      }
      Value::Object(object) => {
        for (k, v) in &mut object.properties {
          k.visit_values_mut(visit);
          v.visit_values_mut(visit);
        }
      }
      Value::Void => {}
      Value::Undefined => {}
      Value::Null => {}
      Value::Bool(..) => {}
      Value::Number(..) => {}
      Value::BigInt(..) => {}
      Value::String(..) => {}
      Value::Register(..) => {}
      Value::Pointer(..) => {}
      Value::Builtin(..) => {}
    }
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::Void => write!(f, "void"),
      Value::Undefined => write!(f, "undefined"),
      Value::Null => write!(f, "null"),
      Value::Bool(value) => write!(f, "{}", value),
      Value::Number(Number(value)) => {
        if value.is_infinite() {
          if value.is_sign_positive() {
            write!(f, "Infinity")
          } else {
            write!(f, "-Infinity")
          }
        } else {
          write!(f, "{}", value)
        }
      }
      Value::BigInt(value) => write!(f, "{}n", value),
      Value::String(value) => write!(
        f,
        "{}",
        serde_json::to_string(&value).expect("Failed json serialization")
      ),
      Value::Array(value) => write!(f, "{}", value),
      Value::Object(value) => write!(f, "{}", value),
      Value::Register(value) => write!(f, "{}", value),
      Value::Pointer(value) => write!(f, "{}", value),
      Value::Builtin(value) => write!(f, "{}", value),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Lazy {
  pub body: Vec<FnLine>,
}

impl std::fmt::Display for Lazy {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "lazy {{\n")?;

    for fn_line in &self.body {
      match fn_line {
        FnLine::Instruction(instruction) => write!(f, "  {}\n", instruction)?,
        FnLine::Label(label) => write!(f, "{}\n", label)?,
        FnLine::Empty => write!(f, "\n")?,
        FnLine::Comment(message) => write!(f, "  // {}\n", message)?,
      }
    }

    write!(f, "}}")
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Builtin {
  pub name: String,
}

impl std::fmt::Display for Builtin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "${}", self.name)
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
  pub values: Vec<Value>,
}

impl std::fmt::Display for Array {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for (i, value) in self.values.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}", value)?;
    }
    write!(f, "]")
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
  pub properties: Vec<(Value, Value)>,
}

impl std::fmt::Display for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.properties.len() == 0 {
      return write!(f, "{{}}");
    }

    write!(f, "{{ ")?;
    for (i, (key, value)) in self.properties.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}: {}", key, value)?;
    }
    write!(f, " }}")
  }
}

impl Object {
  pub fn try_resolve_key(&self, key: &String) -> Option<&Value> {
    let mut result: Option<&Value> = None;

    for (k, v) in &self.properties {
      if let Value::String(k) = k {
        if k == key {
          result = Some(v);
        }
      } else {
        // If the key is not a string, it's possible that the result we found earlier is overwritten
        // here, so we have to set back to None.
        result = None;
      }
    }

    result
  }
}
