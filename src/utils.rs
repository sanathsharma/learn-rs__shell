use is_executable::IsExecutable;
use std::{env, fs, path::Path};

pub fn get_path() -> Option<String> {
  return match env::var("PATH") {
    Ok(path) => Some(path),
    Err(_) => {
      return None;
    }
  };
}

pub fn find_command(command: &str) -> Option<String> {
  let path = get_path()?;

  for dir in path.split(":") {
    let executable_path_str = format!("{}/{}", dir, command);

    let executable_path = Path::new(executable_path_str.as_str());
    if executable_path.exists() && executable_path.is_executable() {
      return Some(executable_path_str);
    };
  }

  None
}

pub fn find_all_executables() -> Vec<String> {
  let mut executables = Vec::new();
  let path = match get_path() {
    Some(path) => path,
    None => return executables,
  };

  for dir in path.split(":") {
    let items = match fs::read_dir(dir) {
      Ok(items) => items,
      Err(_) => continue,
    };

    for item in items {
      let item = item.ok().unwrap().path();
      if item.is_executable() {
        let file_name = item.file_name().unwrap().to_str().unwrap().to_string();
        executables.push(file_name);
      }
    }
  }

  executables
}

pub fn expand_tilda(path: &&str) -> String {
  match env::var("HOME") {
    Ok(home_path) => path.replace("~", &home_path),
    Err(_) => String::from(*path),
  }
}

/// Splits a vector of strings into groups based on a delimiter.
///
/// # Arguments
///
/// * `vec` - A vector of strings to be split into groups
/// * `delimiter` - The string value that acts as a separator between groups
///
/// # Returns
///
/// A vector of string vectors where each inner vector represents a group of strings
/// that were separated by the delimiter in the original vector.
/// Empty groups (those with no elements between delimiters) are not included in the result.
pub fn split_vec_by_delimiter(vec: Vec<String>, delimiter: &str) -> Vec<Vec<String>> {
  let mut result = Vec::new();
  let mut current_group = Vec::new();

  for item in vec {
    if item == delimiter {
      if !current_group.is_empty() {
        result.push(current_group);
        current_group = Vec::new();
      }
    } else {
      current_group.push(item);
    }
  }

  // Push the last group
  if !current_group.is_empty() {
    result.push(current_group);
  }

  result
}
