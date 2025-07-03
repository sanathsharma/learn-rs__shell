use crate::trie::Trie;
use crate::utils::find_all_executables;

pub fn setup_completions() -> Trie {
  let mut completions = Trie::new();
  completions.insert("echo");
  completions.insert("exit");
  completions.insert("type");

  for executable in find_all_executables() {
    completions.insert(&executable);
  }

  completions
}

#[derive(Default, Debug)]
pub struct TabCompletions {
  enabled: bool,
  pub completions: Vec<String>,
}

impl TabCompletions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn reset(&mut self) {
    self.enabled = false;
    self.completions.clear();
  }

  pub fn enable(&mut self) {
    self.enabled = true;
  }

  pub fn set_completions(&mut self, completions: Vec<String>) {
    self.completions = completions;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }
}
