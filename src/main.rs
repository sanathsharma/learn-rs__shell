// Bash impl docs, see https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output

use crate::trie::Trie;
use args::parse_args;
use command::Cmd;
use std::io::Read;
use std::io::{self, Write};
use std::process::Command;

mod ansi_codes;
mod args;
mod command;
mod error;
mod trie;
mod utils;
mod writer;

use utils::find_all_executables;
use ansi_codes::AnsiCode;
pub use error::Result;

fn main() -> Result<()> {
  loop {
    let mut completions = setup_completions();
    print!("$ ");
    io::stdout().flush()?;

    // Wait for user input
    // let mut input = String::new();
    // io::stdin().read_line(&mut input)?;
    let input = match read_input(&mut completions)? {
      Some(input) => input,
      None => continue,
    };

    let cmd_args = parse_args(input.trim().to_string());
    // println!(">>> args: {:?}", cmd_args);

    match Cmd::from(cmd_args.args[0].clone()) {
      Cmd::Unknown => {
        println!("{}: command not found", input.trim());
      }
      command => {
        command.exec(cmd_args);
        continue;
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
        print!("\r\n{}\n\r$ {}", tab_completions.completions.join("  "),String::from_utf8_lossy(&input) );
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

            AnsiCode::BEL.write();
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
