#[allow(unused_imports)]
use std::io::{self, Write};

use command::Cmd;

mod command;

fn main() {
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let parts = input.trim().split_whitespace().collect::<Vec<&str>>();
    match Cmd::from(parts[0]) {
      Cmd::Unknown => {
        println!("{}: command not found", input.trim());
      }
      command => {
        command.exec(parts);
        continue;
      }
    }
  }
}
