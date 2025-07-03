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

// pub struct HistoryCompletions {
//   pointer: usize,
//   completions: Trie,
// }
