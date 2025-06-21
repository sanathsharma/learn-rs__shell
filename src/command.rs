use is_executable::IsExecutable;
use std::{env, io::{self, Write}, path::Path, process};

pub struct ExecutableCmd {
  cmd: String,
  path: String,
}

pub enum Cmd {
  Exit,
  Echo,
  Type,
  Executable(ExecutableCmd),
  Unknown,
}

impl From<&str> for Cmd {
  fn from(value: &str) -> Self {
    match value {
      "echo" => Cmd::Echo,
      "exit" => Cmd::Exit,
      "type" => Cmd::Type,
      cmd => {
        if let Some(executable_path) = find_command(cmd) {
          return Cmd::Executable(ExecutableCmd {
            cmd: cmd.to_string(),
            path: executable_path,
          });
        };

        Cmd::Unknown
      }
    }
  }
}

impl Cmd {
  pub fn exec(&self, parts: Vec<&str>) {
    match self {
      Self::Exit => exec_exit(parts),
      Self::Echo => exec_echo(parts),
      Self::Type => exec_type(parts),
      Self::Executable(cmd) => exec_executable(cmd, parts),
      Self::Unknown => {}
    }
  }
}

fn exec_exit(parts: Vec<&str>) {
  match parts.as_slice() {
    ["exit"] => process::exit(255),
    ["exit", code] => {
      if let Ok(code) = code.parse::<u8>() {
        process::exit(code.into());
      }

      println!("exit: invalid code");
    }
    _ => println!("exit: expected 1 arg at most"),
  }
}

fn exec_echo(parts: Vec<&str>) {
  let args = &parts[1..].join(" ");
  println!("{}", args);
}

fn find_command(command: &str) -> Option<String> {
  let path = match env::var("PATH") {
    Ok(path) => path,
    Err(_) => {
      return None;
    }
  };

  for dir in path.split(":") {
    let executable_path_str = format!("{}/{}", dir, command);

    let executable_path = Path::new(executable_path_str.as_str());
    if executable_path.exists() && executable_path.is_executable() {
      return Some(executable_path_str);
    };
  }

  None
}

fn exec_type(parts: Vec<&str>) {
  match parts.as_slice() {
    ["type", command] => {
      let builtin = Cmd::from(*command);
      match builtin {
        Cmd::Unknown => println!("{}: not found", command),
        Cmd::Executable(exe) => println!("{} is {}", exe.cmd, exe.path),
        _ => println!("{} is a shell builtin", command),
      }
    }
    _ => println!("type: expected 1 arg"),
  }
}

fn exec_executable(executable_cmd: &ExecutableCmd, parts: Vec<&str>) {
  let command = std::process::Command::new(executable_cmd.cmd.clone())
    .args(parts.iter().skip(1))
    .spawn();

  let output = match command {
    Ok(child) => {
      child.wait_with_output()
    },
    Err(_) => {
      println!("{}: failed to execute", executable_cmd.cmd);
      return;
    },
  };

  match output {
    Ok(output) => {
      io::stdout().write_all(&output.stdout).unwrap();
    }
    Err(_) => println!("{}: failed to execute", executable_cmd.cmd),
  }
}
