use std::rc::Rc;

use super::vs_value::Val;
use super::vs_undefined::VsUndefined;
use super::vs_number::VsNumber;
use super::operations;
use super::bytecode_decoder::BytecodeDecoder;
use super::instruction::Instruction;

pub struct VirtualMachine {
  pub stack: Vec<StackFrame>,
}

pub struct StackFrame {
  pub decoder: BytecodeDecoder,
  pub registers: Vec<Val>,
  pub this_target: usize,
  pub return_target: usize,
}

impl VirtualMachine {
  pub fn run(&mut self, bytecode: &Rc<Vec<u8>>) -> Val {
    let mut bd = BytecodeDecoder {
      data: bytecode.clone(),
      pos: 0,
    };

    let main_fn = bd.decode_val(&Vec::new());

    if !main_fn.push_frame(self) {
      std::panic!("bytecode does start with function")
    }

    while self.stack.len() > 1 {
      self.step();
    }

    return self.stack[0].registers[0].clone();
  }

  pub fn new() -> VirtualMachine {
    let mut vm = VirtualMachine {
      stack: Default::default(),
    };

    let mut registers: Vec<Val> = Vec::with_capacity(2);
    registers.push(VsUndefined::new());
    registers.push(VsUndefined::new());

    let frame = StackFrame {
      decoder: BytecodeDecoder {
        data: Rc::new(Vec::new()),
        pos: 0,
      },
      registers: registers,
      return_target: 0,
      this_target: 1,
    };

    vm.stack.push(frame);

    return vm;
  }

  pub fn step(&mut self) {
    use Instruction::*;

    let frame = self.stack.last_mut().unwrap();
    
    match frame.decoder.decode_instruction() {
      End => {
        self.pop();
      },

      Mov => {
        let val = frame.decoder.decode_val(&frame.registers);
        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = val;
        }
      },

      OpInc => {
        let register_index = frame.decoder.decode_register_index().unwrap();
        let mut val = frame.registers[register_index].clone();
        val = operations::op_plus(&val, &VsNumber::from_f64(1_f64));
        frame.registers[register_index] = val;
      },

      OpPlus => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_plus(&left, &right);
        }
      },

      OpMul => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_mul(&left, &right);
        }
      },

      OpMod => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_mod(&left, &right);
        }
      },

      OpLess => {
        let left = frame.decoder.decode_val(&frame.registers);
        let right = frame.decoder.decode_val(&frame.registers);

        let register_index = frame.decoder.decode_register_index();

        if register_index.is_some() {
          frame.registers[register_index.unwrap()] = operations::op_less(&left, &right);
        }
      }

      Jmp => {
        let dst = frame.decoder.decode_pos();
        frame.decoder.pos = dst;
      }

      JmpIf => {
        let cond = frame.decoder.decode_val(&frame.registers);
        let dst = frame.decoder.decode_pos();

        if cond.is_truthy() {
          frame.decoder.pos = dst;
        }
      }

      _ => std::panic!("Not implemented"),
    };
  }

  pub fn pop(&mut self) {
    let old_frame = self.stack.pop().unwrap();
    let frame = self.stack.last_mut().unwrap();

    frame.registers[frame.return_target] = old_frame.registers[0].clone();
    frame.registers[frame.this_target] = old_frame.registers[1].clone();
  }
}
