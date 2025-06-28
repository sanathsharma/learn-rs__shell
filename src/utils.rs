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
