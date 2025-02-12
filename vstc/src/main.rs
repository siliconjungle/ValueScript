mod assemble_command;
mod compile_command;
mod handle_diagnostics_cli;
mod resolve_entry_path;
mod run_command;
mod test_inputs;

use std::env;
use std::process::exit;

use assemble_command::assemble_command;
use compile_command::compile_command;
use run_command::run_command;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    std::panic!("Not implemented: ValueScript repl");
  }

  if args.len() == 2 && (args[1] == "-h" || args[1] == "--help" || args[1] == "help") {
    show_help();
    return;
  }

  if args.len() >= 2 && args[1] == "assemble" {
    assemble_command(&args);
    return;
  }

  if args.len() >= 2 && args[1] == "run" {
    run_command(&args);
    return;
  }

  if args.len() >= 2 && args[1] == "compile" {
    compile_command(&args);
    return;
  }

  println!("ERROR: Unrecognized command\n");
  show_help();
  exit(1);
}

fn show_help() {
  println!("ValueScript toolchain 0.1.0");
  println!();
  println!("USAGE:");
  println!("    vstc [OPTIONS] [SUBCOMMAND]");
  println!();
  println!("OPTIONS:");
  println!("    -h, --help");
  println!("            Print help information");
  println!();
  println!("    -V, --version");
  println!("            Print version information");
  println!();
  println!("SUBCOMMANDS:");
  println!("    compile");
  println!("            Compile an entry point");
  println!();
  println!("    assemble");
  println!("            Convert assembly to bytecode");
  println!();
  println!("    disassemble");
  println!("            Convert bytecode to assembly");
  println!();
  println!("    run");
  println!("            Run a program");
  println!();
  println!("    repl");
  println!("            Read Eval Print Loop");
  println!();
  println!("    host");
  println!("            Start database server");
}
