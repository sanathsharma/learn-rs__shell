use is_executable::IsExecutable;
use std::{env, path::Path};

pub fn find_command(command: &str) -> Option<String> {
  let path = match env::var("PATH") {
    Ok(path) => path,
    Err(_) => {
      return None;
    }
  };

  for dir in path.split(":") {
    let executable_path_str = format!("{}/{}", dir, command);

    let executable_path = Path::new(executable_path_str.as_str());
    if executable_path.exists() && executable_path.is_executable() {
      return Some(executable_path_str);
    };
  }

  None
}

pub fn expand_tilda(path: &&str) -> String {
  match env::var("HOME") {
    Ok(home_path) => path.replace("~", &home_path),
    Err(_) => String::from(*path),
  }
}
