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

  /// Finds the longest common prefix path starting from a given prefix.
  ///
  /// This function takes a prefix string and finds the longest unambiguous path in the trie
  /// that extends from this prefix. It's useful for autocomplete and suggestion features.
  ///
  /// # Arguments
  ///
  /// * `prefix` - A string-like type that can be converted to a string reference.
  ///   This is the starting point for finding the longest common prefix.
  ///
  /// # Returns
  ///
  /// * A `String` containing the longest common prefix path.
  ///   - If the prefix doesn't exist in the trie, returns an empty string.
  ///   - If there are multiple possible paths from the prefix, returns the prefix itself.
  ///   - If there's only one possible path, returns the full path to either a leaf node
  ///     or to a node where branching occurs.
  ///
  /// # Examples
  ///
  /// ```
  /// let mut trie = Trie::new();
  /// trie.insert("apple");
  /// trie.insert("application");
  ///
  /// assert_eq!(trie.longest_common_prefix("app"), "app"); // Multiple paths from "app"
  /// assert_eq!(trie.longest_common_prefix("appl"), "apple"); // Only one path from "appl"
  /// ```
  pub fn longest_common_prefix<T: AsRef<str>>(&mut self, prefix: T) -> String {
    let str = prefix.as_ref();
    let mut new_prefix = String::from(str);

    // Handle empty prefix case
    if str.is_empty() {
      return String::new();
    }

    // First phase: Navigate to the node corresponding to the input prefix
    // If we can't find the prefix in the trie, return empty string
    let mut prefix_node = {
      let mut node = &mut self.root;
      for char in str.chars() {
        match node.children.get_mut(&char) {
          Some(n) => node = n,
          None => {
            // Prefix doesn't exist in the trie
            return String::new();
          }
        };
      }
      node
    };

    // Second phase: Continue traversing as long as there's only one unambiguous path
    loop {
      match prefix_node.children.len() {
        // Case 1: Leaf node - we've reached the end of a word
        // Case 2: Multiple children - we've reached a branching point
        // In both cases, we return the current prefix as the LCP
        n if n == 0 || n > 1 => {
          return new_prefix;
        }
        // Case 3: Single child but current node is a word end
        // This means we've reached a complete word that is also a prefix of a longer word
        // We return the current prefix as the LCP to prioritize the complete word
        1 if prefix_node.is_end => {
          return new_prefix;
        }
        // Case 4: Single child and not a word end
        // We can continue extending the prefix along this unambiguous path
        1 => {
          // Get the only child node and its character
          let (char, node) = prefix_node.children.iter_mut().next().unwrap();
          // Move to the child node
          prefix_node = node;
          // Extend the prefix with the new character
          new_prefix.push(*char);
          // Continue the loop to check the next level
          continue;
        }
        // This case should never be reached due to the exhaustive matching above
        _ => {
          break;
        }
      }
    }

    new_prefix
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

  #[test]
  fn test_longest_common_prefix() {
    let mut t = Trie::new();
    t.insert("xyz_foo");
    t.insert("xyz_foo_bar");
    t.insert("xyz_foo_bar_baz");

    assert_eq!(t.longest_common_prefix("xyz_"), "xyz_foo");
    assert_eq!(t.longest_common_prefix("xyz_foo"), "xyz_foo");
    assert_eq!(t.longest_common_prefix("xyz_foo_"), "xyz_foo_bar");
    assert_eq!(t.longest_common_prefix("xyz_foo_bar_"), "xyz_foo_bar_baz");
  }

  #[test]
  fn test_lcp_empty_trie() {
    let mut t = Trie::new();
    assert_eq!(t.longest_common_prefix("any"), "");
    assert_eq!(t.longest_common_prefix(""), "");
  }

  #[test]
  fn test_lcp_branching_paths() {
    let mut t = Trie::new();
    t.insert("apple");
    t.insert("application");
    t.insert("append");
    t.insert("banana");

    // Common prefix "app" followed by different paths
    assert_eq!(t.longest_common_prefix("app"), "app");
    assert_eq!(t.longest_common_prefix("a"), "app");

    // No common prefix beyond "a" for all words
    assert_eq!(t.longest_common_prefix(""), "");

    // Specific path tests
    assert_eq!(t.longest_common_prefix("appl"), "appl");
    assert_eq!(t.longest_common_prefix("appli"), "application");
    assert_eq!(t.longest_common_prefix("appe"), "append");
    assert_eq!(t.longest_common_prefix("b"), "banana");
  }

  #[test]
  fn test_lcp_with_end_markers() {
    let mut t = Trie::new();
    t.insert("a");
    t.insert("ab");
    t.insert("abc");

    // When a prefix is itself a word, it should return that word
    assert_eq!(t.longest_common_prefix("a"), "a");
    assert_eq!(t.longest_common_prefix("ab"), "ab");
    assert_eq!(t.longest_common_prefix("abc"), "abc");
  }

  #[test]
  fn test_lcp_non_existent_prefix() {
    let mut t = Trie::new();
    t.insert("hello");
    t.insert("world");

    // Prefix that doesn't exist in the trie
    assert_eq!(t.longest_common_prefix("hi"), "");
    assert_eq!(t.longest_common_prefix("z"), "");
  }

  #[test]
  fn test_lcp_single_character_words() {
    let mut t = Trie::new();
    t.insert("a");
    t.insert("b");
    t.insert("c");

    assert_eq!(t.longest_common_prefix("a"), "a");
    assert_eq!(t.longest_common_prefix("b"), "b");
    assert_eq!(t.longest_common_prefix("d"), "");
  }

  #[test]
  fn test_lcp_unicode_characters() {
    let mut t = Trie::new();
    t.insert("café");
    t.insert("cafétéria");
    t.insert("caffè");

    assert_eq!(t.longest_common_prefix("caf"), "caf");
    assert_eq!(t.longest_common_prefix("café"), "café");
    assert_eq!(t.longest_common_prefix("cafét"), "cafétéria");
  }
}
