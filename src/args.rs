const SPACE: char = ' ';
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '\"';
const ESCAPE: char = '\\';

use crate::writer::Redirection;

#[derive(Debug)]
pub struct CmdArgs {
  pub args: Vec<String>,
  pub redirection: Redirection,
}

impl CmdArgs {
  pub fn from_args_vec(args: &Vec<String>) -> Self {
    let mut args_iter = args.iter();
    let mut final_args: Vec<String> = Vec::new();
    let mut redirection = Redirection::None;
    loop {
      let current = args_iter.next();
      if let Some(current) = current {
        match current.as_str() {
          "1>" | ">" => {
            redirection = Redirection::Stdout {
              file_path: args_iter.next().unwrap().clone(),
              append: false,
            };
          }
          "2>" => {
            redirection = Redirection::Stderr {
              file_path: args_iter.next().unwrap().clone(),
              append: false,
            }
          }
          "1>>" | ">>" => {
            redirection = Redirection::Stdout {
              file_path: args_iter.next().unwrap().clone(),
              append: true,
            };
          }
          "2>>" => {
            redirection = Redirection::Stderr {
              file_path: args_iter.next().unwrap().clone(),
              append: true,
            }
          }
          _ => {
            final_args.push(current.clone());
          }
        }
      } else {
        break;
      }
    }

    CmdArgs {
      args: final_args,
      redirection,
    }
  }
}

pub enum WaitFor {
  Space,
  SingleQuote,
  DoubleQuote,
}

/// Parses a command line string into individual arguments, handling single-quoted strings.
///
/// This function splits the input string on spaces while preserving quoted arguments.
/// Single quotes can be used to group words with spaces into a single argument.
///
/// # Arguments
/// * `full_command` - The complete command line string to parse
///
/// # Returns
/// A vector of strings, where each string is a separate command argument
pub fn parse_args(full_command: String) -> CmdArgs {
  let mut args: Vec<String> = Vec::new();
  let mut arg = String::new();
  // Wait for this char while appending other characters to arg
  let mut wait_for = WaitFor::Space;
  let mut is_escaping = false;

  for char in full_command.chars() {
    if is_escaping {
      match wait_for {
        // Escaping outside quotes (non-quoted backlash, preserves the literal value of next char)
        WaitFor::Space => {
          arg.push(char);
        }
        // with single quotes, every char is treaded literally and no escaping is performed
        WaitFor::SingleQuote => {
          arg.push(ESCAPE);
          arg.push(char);
        }
        WaitFor::DoubleQuote => match char {
          ESCAPE | DOUBLE_QUOTE => arg.push(char),
          // with double quotes, every char other the selected few is treaded literally and no escaping is performed
          _ => {
            arg.push(ESCAPE);
            arg.push(char);
          }
        },
      }
      is_escaping = false;
      continue;
    }

    match char {
      SPACE => {
        match wait_for {
          WaitFor::Space => {
            // Skip consecutive spaces
            if arg.is_empty() {
              continue;
            }
            // End of current argument - add it to the list
            args.push(arg.clone());
            arg.clear();
          }
          // If we're inside quotes, treat space as a regular character
          WaitFor::SingleQuote | WaitFor::DoubleQuote => {
            arg.push(SPACE);
            continue;
          }
        }
      }
      SINGLE_QUOTE => {
        match wait_for {
          // Start of quoted string - begin collecting characters until closing quote
          WaitFor::Space => wait_for = WaitFor::SingleQuote,
          // End of quoted string - change the wait_for to space. push arg to args only on space
          WaitFor::SingleQuote => wait_for = WaitFor::Space,
          // In between double quotes - add it to the current argument
          WaitFor::DoubleQuote => arg.push(SINGLE_QUOTE),
        }
      }
      DOUBLE_QUOTE => {
        match wait_for {
          // Start of quoted string - begin collecting characters until closing quote
          WaitFor::Space => wait_for = WaitFor::DoubleQuote,
          // End of quoted string - change the wait_for to space. push arg to args only on space
          WaitFor::DoubleQuote => wait_for = WaitFor::Space,
          // In between single quotes - add it to the current argument
          WaitFor::SingleQuote => arg.push(DOUBLE_QUOTE),
        }
      }
      ESCAPE => is_escaping = true,
      // Regular character - add it to the current argument
      ch => arg.push(ch),
    }
  }

  if !arg.is_empty() {
    args.push(arg);
  }

  CmdArgs::from_args_vec(&args)
}
