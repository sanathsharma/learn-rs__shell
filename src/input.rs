use crate::ansi_codes::AnsiCode;
use crate::history::{History, HistoryNavigation};
use crate::tab_completions::TabCompletionsCtx;
use crate::trie::Trie;
use std::io;
use std::io::{Read, Write};
use std::process::Command;

enum SequenceState {
  Normal,
  ESCReceived,
  BracketReceived,
}

pub fn read_input(cmd_completions: &mut Trie, history: &History) -> crate::Result<Option<String>> {
  let mut buf = [0u8; 1];
  let mut input: Vec<u8> = Vec::new();
  let mut stdin = io::stdin();
  let mut stdout = io::stdout();
  let mut tab_completions_ctx = TabCompletionsCtx::new();
  let mut sequence_state = SequenceState::Normal;

  let mut history_nav = HistoryNavigation::from_size(history.stack.len());

  enable_raw_mode()?;

  loop {
    stdin.read_exact(&mut buf)?;

    if tab_completions_ctx.is_enabled() && buf[0] != b'\t' {
      tab_completions_ctx.reset();
    }

    // print!("{:?}-", buf[0]);
    match buf[0] {
      b'\t' if tab_completions_ctx.is_enabled() => {
        sequence_state = SequenceState::Normal;
        print!(
          "\r\n{}\n\r$ {}",
          tab_completions_ctx.completions.join("  "),
          String::from_utf8_lossy(&input)
        );
        stdout.flush()?;
      }
      b'\t' => {
        sequence_state = SequenceState::Normal;
        let mut c = cmd_completions.get_completions(String::from_utf8(input.clone())?);
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
            tab_completions_ctx.enable();
            tab_completions_ctx.set_completions(c);
            let input_clone = input.clone();
            let prefix = String::from_utf8_lossy(input_clone.as_slice());
            let lcp = cmd_completions.longest_common_prefix(&prefix);

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
        sequence_state = SequenceState::Normal;
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
      // ESC
      27 if matches!(sequence_state, SequenceState::Normal) => {
        sequence_state = SequenceState::ESCReceived;
      }
      // ESC [
      91 if matches!(sequence_state, SequenceState::ESCReceived) => {
        sequence_state = SequenceState::BracketReceived;
      }
      // Up arrow [27, 91, 65] or "ESC [ 65"
      65 if matches!(sequence_state, SequenceState::BracketReceived) => {
        sequence_state = SequenceState::Normal;
        if let Some(completion) = history_nav.previous(&history.stack) {
          print!("\r\x1b[K"); // Clear line and move cursor to start
          print!("$ {}", completion);
          stdout.flush()?; // Push all changes to stdout immediately
        }
      }
      // Down arrow [27, 91, 66] or "ESC [ 66"
      66 if matches!(sequence_state, SequenceState::BracketReceived) => {
        sequence_state = SequenceState::Normal;
        if let Some(completion) = history_nav.next(&history.stack) {
          print!("\r\x1b[K"); // Clear line and move cursor to start
          print!("$ {}", completion);
          stdout.flush()?; // Push all changes to stdout immediately
        } else {
          print!("\r\x1b[K"); // Clear line and move cursor to start
          print!("$ {}", String::from_utf8_lossy(&input));
          stdout.flush()?; // Push all changes to stdout immediately
        };
      }
      o => {
        sequence_state = SequenceState::Normal;
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
