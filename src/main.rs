#![allow(dead_code)]
// Bash impl docs, see https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output

use std::env;
use args::parse_args;
use command::Cmd;
use std::io::{self, Write};
use std::process::Stdio;

mod ansi_codes;
mod args;
mod command;
mod error;
mod history;
mod input;
mod tab_completions;
mod trie;
mod utils;
mod writer;

use crate::command::{CmdInput, ExecutionOutput};
use crate::history::History;
use crate::input::read_input;
use crate::tab_completions::setup_cmd_completions;
use crate::writer::{CmdOutput, CmdOutputWriter, Redirection};
pub use error::Result;

/// Main entry point for the shell implementation.
///
/// This function implements a basic shell with support for:
/// - Command execution
/// - Piping between commands
/// - Input/output redirection
/// - Command history and tab completion
///
/// The shell runs in an infinite loop, continuously prompting for and processing user input
/// until explicitly terminated (e.g., with the "exit" command).
fn main() -> Result<()> {
  let mut history = History::new();

  if let Ok(histfile) = env::var("HISTFILE") {
    history.set_from_file(&histfile);
  }

  loop {
    // Set up command completion for better user experience
    let mut cmd_completions = setup_cmd_completions();

    // Display the shell prompt
    print!("$ ");
    io::stdout().flush()?;
    io::stderr().flush()?;

    // Wait for user input
    // let mut input = String::new();
    // io::stdin().read_line(&mut input)?;
    let input = match read_input(&mut cmd_completions, &history)? {
      Some(input) => input,
      None => continue,
    };

    // Push new command input into history stack
    history.push(&input);

    // Skip empty input lines
    if input.trim().is_empty() {
      continue;
    }

    // Parse the input into a list of commands and their redirections
    let cmds_list = parse_args(input.trim().to_string());
    let len = cmds_list.len();

    // Variable to hold piped input between commands
    let mut piped_stdin: Option<CmdInput> = None;

    // Process each command in the pipeline
    for (index, (cmd_args, redirection)) in cmds_list.iter().enumerate() {
      // Check if this command's output should be piped to the next command
      let is_piped = index < len - 1;

      // Execute the command
      let execution_output = match Cmd::from(cmd_args[0].clone()) {
        Cmd::Unknown => {
          println!("{}: command not found", input.trim());
          ExecutionOutput::none()
        }
        command => command.exec(cmd_args.to_vec(), piped_stdin.take(), &mut history),
      };

      // Handle the command output based on redirection and piping
      match (execution_output, redirection) {
        // First match arm: Handles piping between commands
        // This arm matches when:
        // 1. The command produced stdout output (Some(stdout))
        // 2. There was no stderr output (None)
        // 3. Either no redirection was specified or only stderr redirection was specified
        // 4. This is not the last command in the pipeline (is_piped is true)
        (ExecutionOutput(Some(stdout), None), Redirection::None | Redirection::Stderr { .. })
          if is_piped =>
        {
          match stdout {
            // If the output is a string, convert it to CmdInput::String for the next command
            CmdOutput::Stdout(string) => {
              // Pass the string output to the next command's stdin
              piped_stdin = Some(CmdInput::String(string));
            }
            // If the output is binary data, convert it to CmdInput::Bytes for the next command
            CmdOutput::StdoutBytes(bytes) => {
              // Pass the binary output to the next command's stdin
              piped_stdin = Some(CmdInput::Bytes(bytes));
            }
            CmdOutput::Stream(mut child) => {
              let stdout = child.stdout.take().unwrap();
              piped_stdin = Some(CmdInput::Pipe(Stdio::from(stdout)));
            }
            // Ignore other output types for piping
            _ => {}
          }
        }
        // Second match arm: Catch-all for all other cases
        // This handles:
        // 1. The last command in the pipeline (where output goes to terminal or file)
        // 2. Commands with explicit stdout redirection
        // 3. Commands that produced stderr output
        (execution_output, redirection) => {
          // Write the output according to the redirection rules
          // This handles writing to files or the terminal based on redirection settings
          let _ = write_execution_output(redirection.clone(), execution_output);
        }
      }
    }
  }
}

fn write_execution_output(redirection: Redirection, execution_output: ExecutionOutput) {
  let writer = CmdOutputWriter::new(redirection);
  let ExecutionOutput(stdout, stderr) = execution_output;

  if let Some(stdout) = stdout {
    writer.write_cmd_output(stdout);
  }

  if let Some(stderr) = stderr {
    writer.write_cmd_output(stderr);
  }
}
