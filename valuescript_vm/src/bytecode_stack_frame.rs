use std::mem::take;

use valuescript_common::InstructionByte;

use crate::builtins::type_error_builtin::ToTypeError;
use crate::bytecode_decoder::BytecodeDecoder;
use crate::bytecode_decoder::BytecodeType;
use crate::cat_stack_frame::CatStackFrame;
use crate::native_function::ThisWrapper;
use crate::operations;
use crate::stack_frame::FrameStepOk;
use crate::stack_frame::FrameStepResult;
use crate::stack_frame::{CallResult, StackFrame, StackFrameTrait};
use crate::vs_object::VsObject;
use crate::vs_value::ToVal;
use crate::vs_value::{LoadFunctionResult, Val, ValTrait};

#[derive(Clone)]
pub struct BytecodeStackFrame {
  pub decoder: BytecodeDecoder,
  pub registers: Vec<Val>,
  pub const_this: bool,
  pub param_start: usize,
  pub param_end: usize,
  pub this_target: Option<usize>,
  pub return_target: Option<usize>,
  pub catch_setting: Option<CatchSetting>,
}

#[derive(Clone)]
pub struct CatchSetting {
  pub pos: usize,
  pub register: Option<usize>,
}

impl BytecodeStackFrame {
  pub fn apply_unary_op(&mut self, op: fn(input: &Val) -> Val) {
    let input = self.decoder.decode_val(&mut self.registers);

    let register_index = self.decoder.decode_register_index();

    if register_index.is_some() {
      self.registers[register_index.unwrap()] = op(&input);
    }
  }

  pub fn apply_binary_op(
    &mut self,
    op: fn(left: &Val, right: &Val) -> Result<Val, Val>,
  ) -> Result<(), Val> {
    let left = self.decoder.decode_val(&mut self.registers);
    let right = self.decoder.decode_val(&mut self.registers);

    if let Some(register_index) = self.decoder.decode_register_index() {
      self.registers[register_index] = op(&left, &right)?;
    }

    Ok(())
  }

  pub fn transfer_parameters(&mut self, new_frame: &mut StackFrame) {
    let bytecode_type = self.decoder.peek_type();

    if bytecode_type == BytecodeType::Array {
      self.decoder.decode_type();

      while self.decoder.peek_type() != BytecodeType::End {
        let p = self.decoder.decode_val(&mut self.registers);
        new_frame.write_param(p);
      }

      self.decoder.decode_type(); // End (TODO: assert)

      return;
    }

    let params = self.decoder.decode_val(&mut self.registers);

    match params {
      Val::Array(array_data) => {
        for param in &array_data.elements {
          new_frame.write_param(param.clone())
        }
      }
      _ => panic!("Unexpected non-array params"),
    }
  }

  pub fn decode_parameters(&mut self) -> Vec<Val> {
    let mut res = Vec::<Val>::new();

    let bytecode_type = self.decoder.peek_type();

    if bytecode_type == BytecodeType::Array {
      self.decoder.decode_type();

      while self.decoder.peek_type() != BytecodeType::End {
        res.push(self.decoder.decode_val(&mut self.registers));
      }

      self.decoder.decode_type(); // End (TODO: assert)

      return res;
    }

    let params = self.decoder.decode_val(&mut self.registers);

    match params {
      Val::Array(array_data) => array_data.elements.clone(),
      _ => panic!("Unexpected non-array params"),
    }
  }
}

impl StackFrameTrait for BytecodeStackFrame {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val> {
    self.registers[1] = this;
    self.const_this = const_;
    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    if self.param_start < self.param_end {
      self.registers[self.param_start] = param;
      self.param_start += 1;
    }
  }

  fn step(&mut self) -> FrameStepResult {
    use InstructionByte::*;

    let instruction_byte = self.decoder.decode_instruction();

    match instruction_byte {
      End => {
        return Ok(FrameStepOk::Pop(CallResult {
          return_: take(&mut self.registers[0]),
          this: take(&mut self.registers[1]),
        }));
      }

      Mov => {
        let val = self.decoder.decode_val(&mut self.registers);
        let register_index = self.decoder.decode_register_index();

        if register_index.is_some() {
          self.registers[register_index.unwrap()] = val;
        }
      }

      OpInc => {
        let register_index = self.decoder.decode_register_index().unwrap();
        let val = &mut self.registers[register_index];

        match val {
          Val::Number(n) => *n += 1.0,
          Val::BigInt(bi) => *bi += 1,
          _ => *val = operations::op_plus(val, &1.0.to_val())?,
        };
      }

      OpDec => {
        let register_index = self.decoder.decode_register_index().unwrap();
        let val = &mut self.registers[register_index];

        match val {
          Val::Number(n) => *n -= 1.0,
          Val::BigInt(bi) => *bi -= 1,
          _ => *val = operations::op_minus(val, &1.0.to_val())?,
        };
      }

      OpPlus => self.apply_binary_op(operations::op_plus)?,
      OpMinus => self.apply_binary_op(operations::op_minus)?,
      OpMul => self.apply_binary_op(operations::op_mul)?,
      OpDiv => self.apply_binary_op(operations::op_div)?,
      OpMod => self.apply_binary_op(operations::op_mod)?,
      OpExp => self.apply_binary_op(operations::op_exp)?,
      OpEq => self.apply_binary_op(operations::op_eq)?,
      OpNe => self.apply_binary_op(operations::op_ne)?,
      OpTripleEq => self.apply_binary_op(operations::op_triple_eq)?,
      OpTripleNe => self.apply_binary_op(operations::op_triple_ne)?,
      OpAnd => self.apply_binary_op(operations::op_and)?,
      OpOr => self.apply_binary_op(operations::op_or)?,

      OpNot => self.apply_unary_op(operations::op_not),

      OpLess => self.apply_binary_op(operations::op_less)?,
      OpLessEq => self.apply_binary_op(operations::op_less_eq)?,
      OpGreater => self.apply_binary_op(operations::op_greater)?,
      OpGreaterEq => self.apply_binary_op(operations::op_greater_eq)?,
      OpNullishCoalesce => self.apply_binary_op(operations::op_nullish_coalesce)?,
      OpOptionalChain => {
        let mut left = self.decoder.decode_val(&mut self.registers);
        let right = self.decoder.decode_val(&mut self.registers);

        if let Some(register_index) = self.decoder.decode_register_index() {
          self.registers[register_index] = operations::op_optional_chain(&mut left, &right)?;
        }
      }
      OpBitAnd => self.apply_binary_op(operations::op_bit_and)?,
      OpBitOr => self.apply_binary_op(operations::op_bit_or)?,

      OpBitNot => self.apply_unary_op(operations::op_bit_not),

      OpBitXor => self.apply_binary_op(operations::op_bit_xor)?,
      OpLeftShift => self.apply_binary_op(operations::op_left_shift)?,
      OpRightShift => self.apply_binary_op(operations::op_right_shift)?,
      OpRightShiftUnsigned => self.apply_binary_op(operations::op_right_shift_unsigned)?,

      TypeOf => self.apply_unary_op(operations::op_typeof),

      InstanceOf => self.apply_binary_op(operations::op_instance_of)?,
      In => self.apply_binary_op(operations::op_in)?,

      Call => {
        let fn_ = self.decoder.decode_val(&mut self.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            self.return_target = self.decoder.decode_register_index();
            self.this_target = None;

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let res = native_fn(
              ThisWrapper::new(true, &mut Val::Undefined),
              self.decode_parameters(),
            )?;

            match self.decoder.decode_register_index() {
              Some(return_target) => {
                self.registers[return_target] = res;
              }
              None => {}
            };
          }
        };
      }

      Apply => {
        let fn_ = self.decoder.decode_val(&mut self.registers);

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            if self.decoder.peek_type() == BytecodeType::Register {
              self.decoder.decode_type();
              let this_target = self.decoder.decode_register_index();
              self.this_target = this_target;

              if this_target.is_some() {
                new_frame.write_this(false, self.registers[this_target.unwrap()].clone())?;
              }
            } else {
              self.this_target = None;
              new_frame.write_this(true, self.decoder.decode_val(&mut self.registers))?;
            }

            self.transfer_parameters(&mut new_frame);

            self.return_target = self.decoder.decode_register_index();

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(_native_fn) => {
            panic!("Not implemented");
          }
        }
      }

      Bind => {
        let fn_val = self.decoder.decode_val(&mut self.registers);
        let params = self.decoder.decode_val(&mut self.registers);
        let register_index = self.decoder.decode_register_index();

        let params_array = params.as_array_data();

        if params_array.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          panic!("bind params should always be array")
        }

        let bound_fn = fn_val.bind((*params_array.unwrap()).elements.clone());

        if bound_fn.is_none() {
          // Not sure this needs to be an exception in future since compiled
          // code should never violate this
          panic!("fn parameter of bind should always be bindable");
        }

        if register_index.is_some() {
          self.registers[register_index.unwrap()] = bound_fn.unwrap();
        }
      }

      Sub => {
        let mut left = self.decoder.decode_val(&mut self.registers);
        let right = self.decoder.decode_val(&mut self.registers);

        if let Some(register_index) = self.decoder.decode_register_index() {
          self.registers[register_index] = operations::op_sub(&mut left, &right)?;
        }
      }

      SubMov => {
        // TODO: Ideally we would use a reference for the subscript (decode_vallish), but that would
        // be an immutable borrow and it conflicts with the mutable borrow for the target. In
        // theory, this should still be possible because we only need a mutable borrow to an
        // element, not the vec itself. vec.get_many_mut has been considered, but it's not yet
        // stable.
        let subscript = self.decoder.decode_val(&mut self.registers);

        let value = self.decoder.decode_val(&mut self.registers);

        let target_index = self.decoder.decode_register_index().unwrap();

        operations::op_submov(&mut self.registers[target_index], &subscript, value)?;
      }

      SubCall | ConstSubCall | ThisSubCall => {
        let const_call = instruction_byte == InstructionByte::ConstSubCall
          || (instruction_byte == InstructionByte::ThisSubCall && self.const_this);

        let mut obj = match self.decoder.peek_type() {
          BytecodeType::Register => {
            self.decoder.decode_type();

            ThisArg::Register(self.decoder.decode_register_index().unwrap())
          }
          _ => ThisArg::Val(self.decoder.decode_val(&mut self.registers)),
        };

        let subscript = self.decoder.decode_val(&mut self.registers);

        let fn_ = match &obj {
          ThisArg::Register(reg_i) => self.registers[*reg_i].sub(&subscript)?,
          ThisArg::Val(val) => val.sub(&subscript)?,
        };

        match fn_.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err("fn_ is not a function".to_type_error());
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            self.transfer_parameters(&mut new_frame);

            new_frame.write_this(
              const_call,
              match &obj {
                ThisArg::Register(reg_i) => take(&mut self.registers[reg_i.clone()]),
                ThisArg::Val(val) => val.clone(),
              },
            )?;

            self.return_target = self.decoder.decode_register_index();

            self.this_target = match obj {
              ThisArg::Register(reg_i) => Some(reg_i),
              ThisArg::Val(_) => None,
            };

            return Ok(FrameStepOk::Push(new_frame));
          }
          LoadFunctionResult::NativeFunction(native_fn) => {
            let params = self.decode_parameters();

            let res = match &mut obj {
              ThisArg::Register(reg_i) => native_fn(
                ThisWrapper::new(const_call, self.registers.get_mut(reg_i.clone()).unwrap()),
                params,
              )?,
              ThisArg::Val(val) => native_fn(ThisWrapper::new(true, val), params)?,
            };

            match self.decoder.decode_register_index() {
              Some(return_target) => {
                self.registers[return_target] = res;
              }
              None => {}
            };
          }
        };
      }

      Jmp => {
        let dst = self.decoder.decode_pos();
        self.decoder.pos = dst;
      }

      JmpIf => {
        let cond = self.decoder.decode_val(&mut self.registers);
        let dst = self.decoder.decode_pos();

        if cond.is_truthy() {
          self.decoder.pos = dst;
        }
      }

      UnaryPlus => self.apply_unary_op(operations::op_unary_plus),
      UnaryMinus => self.apply_unary_op(operations::op_unary_minus),

      New => {
        // TODO: new Array

        let class = match self.decoder.decode_val(&mut self.registers).as_class_data() {
          Some(class) => class,
          None => {
            return Err("value is not a constructor".to_type_error());
          }
        };

        let mut instance = VsObject {
          string_map: Default::default(),
          symbol_map: Default::default(),
          prototype: Some(class.instance_prototype.clone()),
        }
        .to_val();

        match class.constructor {
          Val::Void => {
            // Ignore parameters
            self.decoder.decode_val(&mut self.registers);
            let target_register = self.decoder.decode_register_index();

            match target_register {
              None => {}
              Some(tr) => self.registers[tr] = instance,
            };
          }
          _ => match class.constructor.load_function() {
            LoadFunctionResult::NotAFunction => {
              return Err("fn_ is not a function".to_type_error());
            }
            LoadFunctionResult::StackFrame(mut new_frame) => {
              self.transfer_parameters(&mut new_frame);
              new_frame.write_this(false, instance)?;

              self.return_target = None;
              self.this_target = self.decoder.decode_register_index();

              return Ok(FrameStepOk::Push(new_frame));
            }
            LoadFunctionResult::NativeFunction(native_fn) => {
              native_fn(
                ThisWrapper::new(false, &mut instance),
                self.decode_parameters(),
              )?;

              match self.decoder.decode_register_index() {
                Some(target) => {
                  self.registers[target] = instance;
                }
                None => {}
              };
            }
          },
        };
      }

      Throw => {
        return match self.decoder.peek_type() {
          BytecodeType::Register => {
            self.decoder.decode_type();

            // Avoid the void->undefined conversion here
            let error = self.registers[self.decoder.decode_register_index().unwrap()].clone();

            match error {
              Val::Void => Ok(FrameStepOk::Continue),
              _ => Err(error),
            }
          }
          _ => Err(self.decoder.decode_val(&mut self.registers)),
        };
      }

      Import | ImportStar => {
        panic!("TODO: Dynamic imports")
      }

      SetCatch => {
        self.catch_setting = Some(CatchSetting {
          pos: self.decoder.decode_pos(),
          register: self.decoder.decode_register_index(),
        });
      }

      UnsetCatch => {
        self.catch_setting = None;
      }

      RequireMutableThis => {
        if self.const_this {
          return Err("Cannot mutate this because it is const".to_type_error());
        }
      }

      Next => {
        let iter_i = match self.decoder.decode_register_index() {
          Some(i) => i,
          None => panic!("The ignore register is not iterable"),
        };

        let res_i = self.decoder.decode_register_index();

        let next_fn = self.registers[iter_i].sub(&"next".to_val())?;

        match next_fn.load_function() {
          LoadFunctionResult::NotAFunction => {
            return Err(".next() is not a function".to_type_error())
          }
          LoadFunctionResult::NativeFunction(fn_) => {
            let res = fn_(ThisWrapper::new(false, &mut self.registers[iter_i]), vec![])?;

            if let Some(res_i) = res_i {
              self.registers[res_i] = res;
            }
          }
          LoadFunctionResult::StackFrame(mut new_frame) => {
            new_frame.write_this(false, self.registers[iter_i].clone())?;

            self.return_target = res_i;
            self.this_target = Some(iter_i);

            return Ok(FrameStepOk::Push(new_frame));
          }
        };
      }

      UnpackIterRes => {
        let iter_res_i = match self.decoder.decode_register_index() {
          Some(i) => i,
          None => panic!("Can't unpack the ignore register"),
        };

        if let Some(value_i) = self.decoder.decode_register_index() {
          self.registers[value_i] = self.registers[iter_res_i].sub(&"value".to_val())?;
        }

        if let Some(done_i) = self.decoder.decode_register_index() {
          self.registers[done_i] = self.registers[iter_res_i].sub(&"done".to_val())?;
        }
      }

      Cat => {
        assert!(
          self.decoder.decode_type() == BytecodeType::Array,
          "TODO: cat non-inline arrays"
        );

        let cat_frame = CatStackFrame::from_args(self.decoder.decode_vec_val(&mut self.registers));

        self.this_target = None;
        self.return_target = self.decoder.decode_register_index();

        return Ok(FrameStepOk::Push(Box::new(cat_frame)));
      }

      Yield => {
        let val = self.decoder.decode_val(&mut self.registers);
        self.decoder.decode_register_index(); // TODO: Use this

        return Ok(FrameStepOk::Yield(val));
      }

      YieldStar => {
        let val = self.decoder.decode_val(&mut self.registers);
        self.decoder.decode_register_index(); // TODO: Use this

        return Ok(FrameStepOk::YieldStar(val));
      }
    };

    Ok(FrameStepOk::Continue)
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    match self.this_target {
      None => {}
      Some(tt) => {
        self.registers[tt] = call_result.this;
      }
    };

    match self.return_target {
      None => {}
      Some(rt) => {
        self.registers[rt] = call_result.return_;
      }
    };
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for BytecodeStackFrame")
  }

  fn catch_exception(&mut self, exception: Val) -> bool {
    if let Some(catch_setting) = &self.catch_setting {
      if let Some(r) = catch_setting.register {
        self.registers[r] = exception;
      }

      self.decoder.pos = catch_setting.pos;
      self.catch_setting = None;

      true
    } else {
      false
    }
  }

  fn clone_to_stack_frame(&self) -> StackFrame {
    Box::new(self.clone())
  }
}

enum ThisArg {
  Register(usize),
  Val(Val),
}
