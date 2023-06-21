use std::{
  collections::HashSet,
  env, fs,
  path::PathBuf,
  rc::Rc,
  time::{Duration, Instant},
};

use valuescript_compiler::{assemble, compile, resolve_path, ResolvedPath};
use valuescript_vm::{Bytecode, ValTrait, VirtualMachine};

fn main() {
  let exe_path = std::env::current_exe().unwrap();
  let mut current_dir = exe_path.parent().unwrap();
  while current_dir.file_name().unwrap() != "target" {
    current_dir = current_dir.parent().unwrap();
  }
  let project_dir = current_dir.parent().unwrap(); // Go up one more level to get the project directory

  let input_dir_path = project_dir.join("inputs");

  let mut failed_paths = HashSet::<PathBuf>::new();

  let mut files =
    get_files_recursively(&input_dir_path.to_path_buf()).expect("Failed to get files");

  files.sort();

  for file_path in files {
    let file_contents = fs::read_to_string(&file_path).expect("Failed to read file contents");

    let first_line = match file_contents.lines().next() {
      Some(first_line) => first_line,
      None => continue,
    };

    if !first_line.starts_with("//! bench()") {
      continue;
    }

    println!(
      "\n{}:",
      file_path
        .strip_prefix(project_dir)
        .unwrap()
        .to_str()
        .unwrap(),
    );

    let resolved_path = resolve_entry_path(
      &file_path
        .to_str()
        .expect("Failed to convert to str")
        .to_string(),
    );

    let compile_result = compile(resolved_path, |path| {
      fs::read_to_string(path).map_err(|err| err.to_string())
    });

    for (path, diagnostics) in compile_result.diagnostics.iter() {
      if diagnostics.len() > 0 {
        dbg!(&path.path, diagnostics);
      }

      for diagnostic in diagnostics {
        use valuescript_compiler::DiagnosticLevel;

        match diagnostic.level {
          DiagnosticLevel::Error | DiagnosticLevel::InternalError => {
            failed_paths.insert(file_path.clone());
          }
          DiagnosticLevel::Lint | DiagnosticLevel::CompilerDebug => {}
        }
      }
    }

    let module = compile_result
      .module
      .expect("Should have exited if module is None");

    let bytecode = Rc::new(Bytecode::new(assemble(&module)));

    let mut vm = VirtualMachine::new();

    let start = Instant::now();

    while Instant::now() - start < Duration::from_secs(1) {
      let before = Instant::now();
      let result = vm.run(bytecode.clone(), None, &[]);
      let after = Instant::now();

      print!("  {}ms", after.duration_since(before).as_millis());

      if let Err(result) = result {
        assert!(false, "{}", result.codify());
      }
    }

    println!();
  }

  if !failed_paths.is_empty() {
    assert!(false, "See failures above");
  }
}

fn get_files_recursively(dir_path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
  let mut files = vec![];

  for entry in fs::read_dir(dir_path)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_file() {
      files.push(path);
    } else if path.is_dir() {
      files.extend(get_files_recursively(&path)?);
    }
  }

  Ok(files)
}

pub fn resolve_entry_path(entry_path: &String) -> ResolvedPath {
  // Like cwd (current working dir), but it's cwd/file.
  // This is a bit of a hack so we can use resolve_path to get the absolute path of the entry point.
  let cwd_file = ResolvedPath {
    path: env::current_dir()
      .expect("Failed to get current directory")
      .as_path()
      .join("file")
      .to_str()
      .expect("Failed to convert to str")
      .to_string(),
  };

  let resolved_entry_path = resolve_path(&cwd_file, entry_path);

  resolved_entry_path
}
