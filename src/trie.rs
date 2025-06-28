use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Trie {
  root: TrieNode,
}

#[derive(Clone, Debug, Default)]
struct TrieNode {
  children: HashMap<char, TrieNode>,
  is_end: bool,
}

impl TrieNode {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn is_leaf(&self) -> bool {
    self.children.is_empty()
  }
}

impl Trie {
  pub fn new() -> Self {
    Self {
      root: TrieNode::new(),
    }
  }

  pub fn insert<T: AsRef<str>>(&mut self, word: T) {
    let str = word.as_ref();
    let len = str.len();
    let mut node = &mut self.root;
    for (index, char) in str.char_indices() {
      let is_end = index + 1 == len;
      // Get new or existing entry
      node = node.children.entry(char).or_default();
      // Update is_end, after new or exiting entry is extracted
      node.is_end = node.is_end || is_end;
    }
  }

  pub fn search<T: AsRef<str>>(&mut self, word: T) -> bool {
    let str = word.as_ref();
    let len = str.len();
    let mut node = &mut self.root;
    for (index, char) in str.char_indices() {
      match node.children.get_mut(&char) {
        Some(n) => node = n,
        None => {
          return false;
        }
      }

      if index + 1 == len && node.is_end {
        return true;
      }
    }

    false
  }

  pub fn starts_with<T: AsRef<str>>(&mut self, prefix: T) -> bool {
    let str = prefix.as_ref();
    let mut node = &mut self.root;
    for char in str.chars() {
      match node.children.get_mut(&char) {
        Some(n) => node = n,
        None => {
          return false;
        }
      }
    }

    true
  }

  /// Given a prefix, return a vector of all words that start with that prefix.
  ///
  /// If the prefix is empty, return an empty vector.
  /// Returned Vector of strings is not sorted.
  pub fn get_completions<T: AsRef<str>>(&mut self, prefix: T) -> Vec<String> {
    let str = prefix.as_ref();
    if str.is_empty() {
      return Vec::new();
    }

    let mut node = &mut self.root;
    for char in str.chars() {
      match node.children.get_mut(&char) {
        Some(n) => node = n,
        None => {
          return Vec::new();
        }
      }
    }

    let mut completions: Vec<String> = Vec::new();
    Self::collect_words(node.deref(), str, &mut completions);

    completions
  }

  fn collect_words(node: &TrieNode, prefix: &str, completions: &mut Vec<String>) {
    if node.is_end {
      completions.push(prefix.to_string());
    }

    for (char, child) in node.children.iter() {
      Self::collect_words(child, &format!("{}{}", prefix, char), completions);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_trie_operations() {
    let mut t = Trie::new();
    t.insert("dont");
    t.insert("can");
    t.insert("c");

    // Test search for partial strings that don't have is_end set to true
    assert!(!t.search("ca"));
    assert!(!t.search("do"));

    // Test starts_with for partial strings
    assert!(t.starts_with("ca"));
    assert!(t.starts_with("do"));

    // Test search for complete words
    assert!(t.search("can"));
    assert!(t.search("dont"));
    assert!(t.search("c"));

    // Test non-existent strings
    assert!(!t.search("cat"));
    assert!(!t.starts_with("b"));
  }

  #[test]
  fn test_empty_trie() {
    let mut t = Trie::new();
    assert!(!t.search(""));
    assert!(t.starts_with(""));
  }

  #[test]
  fn test_insert_empty_string() {
    let mut t = Trie::new();
    t.insert("");
    assert!(!t.search("")); // Empty string is a special case
  }

  #[test]
  fn test_overlapping_prefixes() {
    let mut t = Trie::new();
    t.insert("car");
    t.insert("card");
    t.insert("care");

    assert!(t.search("car"));
    assert!(t.search("card"));
    assert!(t.search("care"));
    assert!(!t.search("ca"));
    assert!(t.starts_with("ca"));
    assert!(t.starts_with("car"));
  }

  #[test]
  fn test_get_completions() {
    let mut t = Trie::new();
    t.insert("car");
    t.insert("card");
    t.insert("care");
    t.insert("carpet");
    t.insert("carrot");
    t.insert("cat");

    // Test completions for "ca"
    let ca_completions = t.get_completions("ca");
    assert!(ca_completions.contains(&"car".to_string()));
    assert!(ca_completions.contains(&"card".to_string()));
    assert!(ca_completions.contains(&"care".to_string()));
    assert!(ca_completions.contains(&"carpet".to_string()));
    assert!(ca_completions.contains(&"carrot".to_string()));
    assert!(ca_completions.contains(&"cat".to_string()));
    assert_eq!(ca_completions.len(), 6);

    // Test completions for "car"
    let car_completions = t.get_completions("car");
    assert!(car_completions.contains(&"car".to_string()));
    assert!(car_completions.contains(&"card".to_string()));
    assert!(car_completions.contains(&"care".to_string()));
    assert!(car_completions.contains(&"carpet".to_string()));
    assert!(car_completions.contains(&"carrot".to_string()));
    assert!(!car_completions.contains(&"cat".to_string()));
    assert_eq!(car_completions.len(), 5);

    // Test completions for "card"
    let card_completions = t.get_completions("card");
    assert!(card_completions.contains(&"card".to_string()));
    assert_eq!(card_completions.len(), 1);

    // Test completions for non-existent prefix
    let non_existent = t.get_completions("z");
    assert!(non_existent.is_empty());

    // Test completions for empty string
    let empty_completions = t.get_completions("");
    assert!(empty_completions.is_empty());
  }
}
