#![allow(dead_code)]
// Bash impl docs, see https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output

use crate::trie::Trie;
use args::parse_args;
use command::Cmd;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

mod ansi_codes;
mod args;
mod command;
mod error;
mod trie;
mod utils;
mod writer;

use crate::command::{CmdInput, ExecutionOutput};
use crate::writer::{CmdOutput, CmdOutputWriter, Redirection};
use ansi_codes::AnsiCode;
pub use error::Result;
use utils::find_all_executables;

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
  loop {
    // Set up command completion for better user experience
    let mut completions = setup_completions();

    // Display the shell prompt
    print!("$ ");
    io::stdout().flush()?;

    // Wait for user input
    // let mut input = String::new();
    // io::stdin().read_line(&mut input)?;
    let input = match read_input(&mut completions)? {
      Some(input) => input,
      None => continue,
    };

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
        command => command.exec(cmd_args.to_vec(), piped_stdin.take()),
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

fn setup_completions() -> Trie {
  let mut completions = Trie::new();
  completions.insert("echo");
  completions.insert("exit");
  completions.insert("type");

  for executable in find_all_executables() {
    completions.insert(&executable);
  }

  completions
}

#[derive(Default, Debug)]
struct TabCompletions {
  enabled: bool,
  pub completions: Vec<String>,
}

impl TabCompletions {
  fn new() -> Self {
    Self::default()
  }

  fn reset(&mut self) {
    self.enabled = false;
    self.completions.clear();
  }

  fn enable(&mut self) {
    self.enabled = true;
  }

  fn set_completions(&mut self, completions: Vec<String>) {
    self.completions = completions;
  }

  fn is_enabled(&self) -> bool {
    self.enabled
  }
}

fn read_input(completions: &mut Trie) -> Result<Option<String>> {
  let mut buf = [0u8; 1];
  let mut input: Vec<u8> = Vec::new();
  let mut stdin = io::stdin();
  let mut stdout = io::stdout();
  let mut tab_completions = TabCompletions::new();

  enable_raw_mode()?;

  loop {
    stdin.read_exact(&mut buf)?;

    if tab_completions.is_enabled() && buf[0] != b'\t' {
      tab_completions.reset();
    }

    match buf[0] {
      b'\t' if tab_completions.is_enabled() => {
        print!(
          "\r\n{}\n\r$ {}",
          tab_completions.completions.join("  "),
          String::from_utf8_lossy(&input)
        );
        stdout.flush()?;
      }
      b'\t' => {
        let mut c = completions.get_completions(String::from_utf8(input.clone())?);
        c.sort();
        match c.len() {
          0 => {
            AnsiCode::BEL.write();
            stdout.flush()?;
          }
          1 => {
            let first_completion = c.first().unwrap();
            let bytes = first_completion.as_bytes();
            print!("\r\x1b[K"); // Clear line and move cursor to start
            print!("$ {} ", first_completion);
            stdout.flush()?; // Push all changes to stdout immediately

            input.clear();
            input.extend(bytes);
            input.extend(b" ");
          }
          _ => {
            tab_completions.enable();
            tab_completions.set_completions(c);
            let input_clone = input.clone();
            let prefix = String::from_utf8_lossy(input_clone.as_slice());
            let lcp = completions.longest_common_prefix(&prefix);

            if prefix == lcp {
              AnsiCode::BEL.write();
              stdout.flush()?;
              continue;
            }

            print!("\r\x1b[K"); // Clear line and move cursor to start
            print!("$ {}", lcp); // No space in the end as multiple completions is available
            stdout.flush()?; // Push all changes to stdout immediately

            input.clear();
            input.extend(lcp.as_bytes());

            stdout.flush()?;
          }
        }
      }
      b'\n' | b'\r' => {
        AnsiCode::CRLF.write();
        break;
      }
      b'\x03' => {
        AnsiCode::CRLF.write();
        return Ok(None);
      }
      // Handle backspace (ASCII 8) and delete (ASCII 127)
      b'\x08' | b'\x7F' => {
        if !input.is_empty() {
          input.pop(); // Remove the last character from input
                       // Move cursor back, erase the character, and move cursor back again
          print!(
            "{}{}{}",
            AnsiCode::MoveCursorLeft,
            " ",
            AnsiCode::MoveCursorLeft
          );
          stdout.flush()?;
        }
      }
      o => {
        print!("{}", String::from_utf8_lossy(&[o]));
        stdout.flush()?;
        input.push(o);
      }
    }
  }

  disable_raw_mode()?;

  Ok(Some(String::from_utf8(input)?))
}

fn enable_raw_mode() -> io::Result<()> {
  Command::new("stty")
    .args(&[
      "raw",   // Raw mode
      "-echo", // Don't echo input, so the shell can decide which chars to echo and which chars are special
      "min", "1", // Return after 1 character
      "time", "0", // No timeout
    ])
    .status()?;
  Ok(())
}

fn disable_raw_mode() -> io::Result<()> {
  Command::new("stty")
    .arg("cooked") // Restore normal mode
    .status()?;
  Ok(())
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
