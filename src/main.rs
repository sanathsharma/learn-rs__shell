// Bash impl docs, see https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output

use crate::trie::Trie;
use args::parse_args;
use command::Cmd;
use std::io::Read;
use std::io::{self, stdout, BufWriter, Cursor, Write};
use std::process::Command;

mod args;
mod command;
mod trie;
mod utils;
mod writer;
mod error;
mod ansi_codes;

pub use error::Result;
use ansi_codes::AnsiCode;

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

  completions
}

fn read_input(completions: &mut Trie) -> Result<Option<String>> {
  let mut buf = [0u8; 1];
  let mut input: Vec<u8> = Vec::new();
  let mut stdin = io::stdin();
  let mut stdout = io::stdout();

  enable_raw_mode()?;

  loop {
    stdin.read_exact(&mut buf)?;
    match buf[0] {
      b'\t' => {
        let c = completions.get_completions(String::from_utf8(input.clone())?);
        if let Some(c) = c.first() {
          let bytes = c.as_bytes();
          print!("\r\x1b[K"); // Clear line and move cursor to start
          print!("$ {} ", c);
          stdout.flush()?; // Push all changes to stdout immediately

          input.clear();
          input.extend(bytes);
          input.extend(b" ");
          continue;
        }

        AnsiCode::BEL.write();
        stdout.flush()?;
      }
      b'\n' | b'\r' => {
        // print!("\n\r"); // Move to start of a new line
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
          print!("{}{}{}", AnsiCode::MoveCursorLeft, " ", AnsiCode::MoveCursorLeft);
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
