use crate::ansi_codes::AnsiCode;
use crate::tab_completions::TabCompletions;
use crate::trie::Trie;
use std::io;
use std::io::{Read, Write};
use std::process::Command;

pub fn read_input(completions: &mut Trie) -> crate::Result<Option<String>> {
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
