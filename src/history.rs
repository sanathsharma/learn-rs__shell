use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

#[derive(Default)]
pub struct History {
  pub stack: Vec<String>,
}

impl History {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_file(file_path: &str) -> Self {
    Self {
      stack: load_file(file_path),
    }
  }

  pub fn push(&mut self, command_str: &str) -> &mut Self {
    self.stack.push(command_str.into());

    self
  }

  // TODO: return a result
  pub fn set_from_file(&mut self, file_path: &str) -> &mut Self {
    let content = load_file(file_path);
    self.stack.clear();
    self.stack.extend(content);

    self
  }

  // TODO: return a result
  pub fn extend_from_file(&mut self, file_path: &str) -> &mut Self {
    let content = load_file(file_path);
    self.stack.extend(content);

    self
  }

  // TODO: return a result
  pub fn write_to_file(&self, file_path: &str, append: bool) {
    let file = OpenOptions::new()
      .write(true)
      .create(true)
      .append(append)
      .truncate(!append)
      .open(file_path);

    let mut file = match file {
      Ok(f) => f,
      Err(_) => {
        return ();
      }
    };

    let mut output = String::new();
    for line in self.stack.iter() {
      output.push_str(format!("{}\n", line).as_str());
    }

    write!(file, "{}", output).unwrap();
  }
}

pub struct HistoryNavigation {
  pointer: usize,
  size: usize,
}

impl HistoryNavigation {
  pub fn from_size(size: usize) -> Self {
    Self {
      pointer: size,
      size,
    }
  }

  pub fn next<'a>(&mut self, stack: &'a Vec<String>) -> Option<&'a String> {
    if self.pointer == self.size {
      return None;
    };

    self.pointer += 1;

    stack.get(self.pointer)
  }

  pub fn previous<'a>(&mut self, stack: &'a Vec<String>) -> Option<&'a String> {
    if self.pointer != 0 {
      self.pointer -= 1;
    }

    stack.get(self.pointer)
  }
}

fn load_file(file_path: &str) -> Vec<String> {
  let file = OpenOptions::new().read(true).open(file_path);
  let file = match file {
    Ok(f) => f,
    Err(_) => return Vec::new(),
  };

  let reader = BufReader::new(file);
  reader.lines().collect::<Result<Vec<String>, _>>().unwrap()
}

// TODO:
// [ ] accept completions and set_completions method
// [ ] given a prefix autocomplete based on the completions from prefix tree
