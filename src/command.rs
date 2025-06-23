use std::{
  env,
  io::{self},
  process::{self, Stdio},
};

use crate::{
  args::CmdArgs,
  utils::{expand_tilda, find_command},
  writer::CmdOuputWriter,
};
pub struct ExecutableCmd {
  cmd: String,

  path: String,
}

pub enum Cmd {
  Exit,
  Echo,
  Type,
  Executable(ExecutableCmd),
  Cd,
  Pwd,
  Unknown,
}

impl From<String> for Cmd {
  fn from(value: String) -> Self {
    match value.as_str() {
      "echo" => Cmd::Echo,
      "exit" => Cmd::Exit,
      "type" => Cmd::Type,
      "pwd" => Cmd::Pwd,
      "cd" => Cmd::Cd,
      cmd => {
        if let Some(executable_path) = find_command(cmd) {
          return Cmd::Executable(ExecutableCmd {
            // avoid conversion from cmd.to_string(), by passing value
            cmd: value,
            path: executable_path,
          });
        };

        Cmd::Unknown
      }
    }
  }
}

impl Cmd {
  pub fn exec(&self, cmd_args: CmdArgs) {
    let writer = CmdOuputWriter::new(cmd_args.redirection.clone());

    match self {
      Self::Exit => exec_exit(cmd_args, writer),
      Self::Echo => exec_echo(cmd_args, writer),
      Self::Type => exec_type(cmd_args, writer),
      Self::Executable(cmd) => exec_executable(cmd, cmd_args, writer),
      Self::Cd => exec_cd(cmd_args, writer),
      Self::Pwd => exec_pwd(cmd_args, writer),
      Self::Unknown => {}
    }
  }
}

fn exec_exit(cmd_args: CmdArgs, writer: CmdOuputWriter) {
  let args = cmd_args
    .args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  match args.as_slice() {
    ["exit"] => process::exit(255),
    ["exit", code] => {
      if let Ok(code) = code.parse::<u8>() {
        process::exit(code.into());
      }

      writer.output_error_string("exit: invalid code");
    }
    _ => writer.output_error_string("exit: expected 1 arg at most"),
  }
}

fn exec_echo(cmd_args: CmdArgs, writer: CmdOuputWriter) {
  let args = &cmd_args.args[1..].join(" ");
  writer.output_string(args);
}

fn exec_type(cmd_args: CmdArgs, writer: CmdOuputWriter) {
  let args = cmd_args
    .args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  match args.as_slice() {
    ["type", command] => {
      let builtin = Cmd::from(command.to_string());
      match builtin {
        Cmd::Unknown => writer.output_error_string(format!("{}: not found", command)),
        Cmd::Executable(exe) => writer.output_string(format!("{} is {}", exe.cmd, exe.path)),
        _ => writer.output_string(format!("{} is a shell builtin", command)),
      }
    }
    _ => writer.output_error_string("type: expected 1 arg"),
  }
}

fn exec_executable(executable_cmd: &ExecutableCmd, cmd_args: CmdArgs, writer: CmdOuputWriter) {
  let args = cmd_args
    .args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  let command = std::process::Command::new(executable_cmd.cmd.clone())
    .args(args.iter().skip(1))
    // INFO: Stdio::piped makes the child not write it to stdout & stderr that is inherited from the
    // terminal session
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn();

  let output = match command {
    Ok(child) => child.wait_with_output(),
    Err(_) => {
      writer.output_error_string(format!("{}: failed to execute", executable_cmd.cmd));
      return;
    }
  };

  match output {
    Ok(output) => {
      if !output.stdout.is_empty() {
        writer.output(&output.stdout);
        return;
      }

      if !output.stderr.is_empty() {
        writer.output_error(&output.stderr);
      }
    }
    Err(_) => writer.output_error_string(format!("{}: failed to execute", executable_cmd.cmd)),
  }
}

fn exec_cd(cmd_args: CmdArgs, writer: CmdOuputWriter) {
  let args = cmd_args
    .args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  let (path, cwd): (&str, io::Result<()>) = match args.as_slice() {
    ["cd"] => ("~", env::set_current_dir(expand_tilda(&"~"))),
    ["cd", path] => {
      if path.starts_with("~") {
        (path, env::set_current_dir(expand_tilda(path)))
      } else {
        (path, env::set_current_dir(path))
      }
    }
    _ => {
      writer.output_error_string("cd: expected 1 arg at most");
      return;
    }
  };

  match cwd {
    Ok(_) => (),
    Err(_) => {
      writer.output_error_string(format!("cd: {}: No such file or directory", path));
    }
  }
}

fn exec_pwd(cmd_args: CmdArgs, writer: CmdOuputWriter) {
  let args = cmd_args
    .args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  match args.as_slice() {
    ["pwd"] => {
      let current_dir = env::current_dir().unwrap();
      writer.output_string(format!("{}", current_dir.display()));
    }
    _ => writer.output_error_string("pwd: expected 0 args"),
  }
}
