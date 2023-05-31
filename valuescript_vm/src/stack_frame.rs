use super::vs_value::Val;

pub type StackFrame = Box<dyn StackFrameTrait>;

#[derive(Clone)]
pub struct CallResult {
  pub return_: Val,
  pub this: Val,
}

pub enum FrameStepOk {
  Continue,
  Pop(CallResult),
  Push(StackFrame),
}

pub type FrameStepResult = Result<FrameStepOk, Val>;

pub trait StackFrameTrait {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val>;
  fn write_param(&mut self, param: Val);
  fn step(&mut self) -> FrameStepResult;
  fn apply_call_result(&mut self, call_result: CallResult);
  fn get_call_result(&mut self) -> CallResult;
  fn catch_exception(&mut self, exception: Val) -> bool;
  fn clone_to_stack_frame(&self) -> StackFrame;
}

impl Clone for StackFrame {
  fn clone(&self) -> Self {
    self.clone_to_stack_frame()
  }
}
