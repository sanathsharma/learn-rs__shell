use crate::writer::CmdOutput;
use crate::{
  args::CmdArgs,
  utils::{expand_tilda, find_command},
};
use std::io::Write;
use std::{
  env,
  io::{self},
  process::{self, Stdio},
};

pub struct ExecutableCmd {
  cmd: String,

  path: String,
}

pub enum CmdInput {
  String(String),
  Bytes(Vec<u8>),
  Pipe(Stdio),
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
  fn from(program: String) -> Self {
    match program.as_str() {
      "echo" => Cmd::Echo,
      "exit" => Cmd::Exit,
      "type" => Cmd::Type,
      "pwd" => Cmd::Pwd,
      "cd" => Cmd::Cd,
      cmd => {
        if let Some(executable_path) = find_command(cmd) {
          return Cmd::Executable(ExecutableCmd {
            // avoid conversion from cmd.to_string(), by passing value
            cmd: program,
            path: executable_path,
          });
        };

        Cmd::Unknown
      }
    }
  }
}

#[derive(Debug)]
pub struct ExecutionOutput(pub Option<CmdOutput>, pub Option<CmdOutput>);

impl ExecutionOutput {
  pub fn none() -> Self {
    Self(None, None)
  }

  pub fn stdout<T: Into<String>>(stdout: T) -> Self {
    Self(Some(CmdOutput::Stdout(stdout.into())), None)
  }

  pub fn stdout_bytes(stdout: Vec<u8>) -> Self {
    Self(Some(CmdOutput::StdoutBytes(stdout)), None)
  }

  pub fn stderr<T: Into<String>>(stderr: T) -> Self {
    Self(None, Some(CmdOutput::Stderr(stderr.into())))
  }

  pub fn stderr_bytes(stderr: Vec<u8>) -> Self {
    Self(None, Some(CmdOutput::StderrBytes(stderr)))
  }
}

impl Cmd {
  pub fn exec(&self, cmd_args: CmdArgs, input: Option<CmdInput>) -> ExecutionOutput {
    match self {
      Self::Exit => exec_exit(cmd_args),
      Self::Echo => exec_echo(cmd_args),
      Self::Type => exec_type(cmd_args),
      Self::Executable(cmd) => exec_executable(cmd, cmd_args, input),
      Self::Cd => exec_cd(cmd_args),
      Self::Pwd => exec_pwd(cmd_args),
      Self::Unknown => ExecutionOutput::none(),
    }
  }
}

fn exec_exit(cmd_args: CmdArgs) -> ExecutionOutput {
  let args = cmd_args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  match args.as_slice() {
    ["exit"] => process::exit(255),
    ["exit", code] => {
      if let Ok(code) = code.parse::<u8>() {
        process::exit(code.into());
      }

      ExecutionOutput::stderr("exit: invalid code")
    }
    _ => ExecutionOutput::stderr("exit: expected 1 arg at most"),
  }
}

fn exec_echo(cmd_args: CmdArgs) -> ExecutionOutput {
  let args = cmd_args[1..].join(" ");
  ExecutionOutput::stdout(args)
}

fn exec_type(cmd_args: CmdArgs) -> ExecutionOutput {
  let args = cmd_args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  match args.as_slice() {
    ["type", command] => {
      let builtin = Cmd::from(command.to_string());
      match builtin {
        Cmd::Unknown => ExecutionOutput::stderr(format!("{}: not found", command)),
        Cmd::Executable(exe) => ExecutionOutput::stdout(format!("{} is {}", exe.cmd, exe.path)),
        _ => ExecutionOutput::stdout(format!("{} is a shell builtin", command)),
      }
    }
    _ => ExecutionOutput::stderr("type: expected 1 arg"),
  }
}

fn exec_executable(
  executable_cmd: &ExecutableCmd,
  cmd_args: CmdArgs,
  input: Option<CmdInput>,
) -> ExecutionOutput {
  let args = cmd_args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  let (stdin, data) = match input {
    Some(CmdInput::Pipe(stdio)) => (stdio, None),
    Some(CmdInput::String(string)) => (Stdio::piped(), Some(CmdInput::String(string))),
    Some(CmdInput::Bytes(bytes)) => (Stdio::piped(), Some(CmdInput::Bytes(bytes))),
    None => (Stdio::inherit(), None),
  };

  let command = std::process::Command::new(executable_cmd.cmd.clone())
    .args(args.iter().skip(1))
    // INFO: Stdio::piped makes the child not write it to stdout & stderr that is inherited from the
    // terminal session
    .stdin(stdin)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn();

  match command {
    Ok(mut child) => {
      match data {
        Some(CmdInput::String(string)) => {
          if let Some(mut stdin) = child.stdin.take() {
            stdin.write(string.as_bytes()).unwrap();
          }
        }
        Some(CmdInput::Bytes(bytes)) => {
          if let Some(mut stdin) = child.stdin.take() {
            stdin.write(bytes.as_slice()).unwrap();
          }
        }
        _ => {}
      }
      ExecutionOutput(Some(CmdOutput::Stream(child)), None)
    }
    Err(_) => {
      return ExecutionOutput::stderr(format!("{}: failed to execute", executable_cmd.cmd));
    }
  }
}

fn exec_cd(cmd_args: CmdArgs) -> ExecutionOutput {
  let args = cmd_args
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
      return ExecutionOutput::stderr("cd: expected 1 arg at most");
    }
  };

  match cwd {
    Ok(_) => ExecutionOutput::none(),
    Err(_) => ExecutionOutput::stderr(format!("cd: {}: No such file or directory", path)),
  }
}

fn exec_pwd(cmd_args: CmdArgs) -> ExecutionOutput {
  let args = cmd_args
    .iter()
    .map(|arg| arg.as_str())
    .collect::<Vec<&str>>();

  match args.as_slice() {
    ["pwd"] => {
      let current_dir = env::current_dir().unwrap();
      ExecutionOutput::stdout(format!("{}", current_dir.display()))
    }
    _ => ExecutionOutput::stderr("pwd: expected 0 args"),
  }
}
