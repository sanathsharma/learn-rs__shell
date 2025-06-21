use std::{env, path::Path, process};
use is_executable::IsExecutable;

pub enum Builtin {
  Exit,
  Echo,
  Type,
  Unknown,
}

impl From<&str> for Builtin {
  fn from(value: &str) -> Self {
    match value {
      "echo" => Builtin::Echo,
      "exit" => Builtin::Exit,
      "type" => Builtin::Type,
      _ => Builtin::Unknown,
    }
  }
}

impl Builtin {
  pub fn exec(&self, parts: Vec<&str>) {
    match self {
      Self::Exit => exec_exit(parts),
      Self::Echo => exec_echo(parts),
      Self::Type => exec_type(parts),
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
    if executable_path.exists() && executable_path.is_executable()  {
      return Some(executable_path_str);
    };
  }

  None
}

fn exec_type(parts: Vec<&str>) {
  match parts.as_slice() {
    ["type", command] => {
      let builtin = Builtin::from(*command);
      match builtin {
        Builtin::Unknown => {
          if let Some(executable_path) = find_command(command) {
            println!("{} is {}", command, executable_path);
            return;
          };
          println!("{}: not found", command);
        }
        _ => println!("{} is a shell builtin", command),
      }
    }
    _ => println!("type: expected 1 arg"),
  }
}
