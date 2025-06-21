use std::process;

pub enum Builtin {
    Exit,
    Echo,
    Unknown,
}

impl From<&str> for Builtin {
    fn from(value: &str) -> Self {
        match value {
            "echo" => Builtin::Echo,
            "exit" => Builtin::Exit,
            _ => Builtin::Unknown,
        }
    }
}

impl Builtin {
    pub fn exec(&self, parts: Vec<&str>) {
        match self {
            Self::Exit => exec_exit(parts),
            Self::Echo => exec_echo(parts),
            Self::Unknown => {}
        }
    }
}

fn exec_exit(parts: Vec<&str>) {
    // When no code is supplied
    if parts.len() == 1 {
        process::exit(255);
    }

    // When a valid code is supplied
    if parts.len() == 2 {
        let code = parts[1].parse::<u8>().expect("exit: invalid code");
        process::exit(code.into());
    }

    println!("exit: expected 1 arg at most");
}

fn exec_echo(parts: Vec<&str>) {
    let args = &parts[1..].join(" ");
    println!("{}", args);
}

