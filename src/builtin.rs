use std::process;

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

fn exec_type(parts: Vec<&str>) {
  match parts.as_slice() {
    ["type", command] => {
      let builtin = Builtin::from(*command);
      match builtin {
        Builtin::Unknown => println!("{}: not found", command),
        _ => println!("{} is a shell builtin", command),
      }
    }
    _ => println!("type: expected 1 arg"),
  }
}
