#[derive(Default)]
pub struct History {
  pub stack: Vec<String>,
}

impl History {
  pub fn new() -> Self {
    Self::default()
  }
  pub fn push(&mut self, command_str: &str) -> &mut Self {
    self.stack.push(command_str.into());

    self
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
