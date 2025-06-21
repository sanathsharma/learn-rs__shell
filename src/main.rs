#[allow(unused_imports)]
use std::io::{self, Write};

use builtin::Builtin;

mod builtin;

fn main() {
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let parts = input.trim().split_whitespace().collect::<Vec<&str>>();
    match Builtin::from(parts[0]) {
      Builtin::Unknown => {}
      builtin => {
        builtin.exec(parts);
        continue;
      }
    }

    println!("{}: command not found", input.trim());
  }
}
