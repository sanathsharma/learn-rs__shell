const SPACE: char = ' ';
const SINGLE_QUOTE: char = '\'';

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
  let mut wait_for = SPACE;

  for char in full_command.chars() {
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
        // Start of quoted string - begin collecting characters until closing quote
        if wait_for != SINGLE_QUOTE {
          wait_for = SINGLE_QUOTE;
          continue;
        }

        // End of quoted string - change the wait_for to space. push arg to args only on space
        wait_for = SPACE;
      }
      // Regular character - add it to the current argument
      ch => arg.push(ch),
    }
  }

  if !arg.is_empty() {
    args.push(arg);
  }

  args
}
