const SPACE: char = ' ';
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '\"';
const ESCAPE: char = '\\';

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
pub fn parse_args(full_command: String) -> Vec<String> {
  let mut args: Vec<String> = Vec::new();
  let mut arg = String::new();
  // Wait for this char while appending other characters to arg
  let mut wait_for = SPACE;
  let mut is_escaping = false;

  for char in full_command.chars() {
    if is_escaping {
      arg.push(char);
      is_escaping = false;
      continue;
    }

    match char {
      SPACE => {
        // If we're inside single quotes, treat space as a regular character
        if wait_for != SPACE {
          arg.push(SPACE);
          continue;
        }
        // Skip consecutive spaces
        if arg.is_empty() {
          continue;
        }
        // End of current argument - add it to the list
        args.push(arg.clone());
        arg.clear();
      }
      SINGLE_QUOTE => {
        match wait_for {
          // Start of quoted string - begin collecting characters until closing quote
          SPACE => wait_for = SINGLE_QUOTE,
          // End of quoted string - change the wait_for to space. push arg to args only on space
          SINGLE_QUOTE => wait_for = SPACE,
          // In between double quotes - add it to the current argument
          DOUBLE_QUOTE => arg.push(SINGLE_QUOTE),
          _ => {}
        }
      }
      DOUBLE_QUOTE => {
        match wait_for {
          // Start of quoted string - begin collecting characters until closing quote
          SPACE => wait_for = DOUBLE_QUOTE,
          // End of quoted string - change the wait_for to space. push arg to args only on space
          DOUBLE_QUOTE => wait_for = SPACE,
          // In between single quotes - add it to the current argument
          SINGLE_QUOTE => arg.push(SINGLE_QUOTE),
          _ => {}
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

  args
}
