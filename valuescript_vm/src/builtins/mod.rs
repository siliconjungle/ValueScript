mod debug_builtin;
mod math_builtin;
mod number_builtin;
mod string_builtin;

use valuescript_common::BUILTIN_COUNT;

use crate::ValTrait;

pub static BUILTIN_VALS: [&'static (dyn ValTrait + Sync); BUILTIN_COUNT] = [
  &debug_builtin::DEBUG_BUILTIN,
  &math_builtin::MATH_BUILTIN,
  &string_builtin::STRING_BUILTIN,
  &number_builtin::NUMBER_BUILTIN,
];
