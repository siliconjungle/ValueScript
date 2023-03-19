mod array_higher_functions;
mod builtins;
mod bytecode_decoder;
mod bytecode_stack_frame;
mod debug;
mod first_stack_frame;
mod helpers;
mod instruction;
mod math;
mod native_frame_function;
mod native_function;
mod number_builtin;
mod number_methods;
mod operations;
mod stack_frame;
mod string_builtin;
mod string_methods;
mod todo_fn;
mod virtual_machine;
mod vs_array;
mod vs_class;
mod vs_function;
mod vs_object;
mod vs_pointer;
mod vs_value;

pub use virtual_machine::VirtualMachine;
pub use vs_value::ValTrait;
