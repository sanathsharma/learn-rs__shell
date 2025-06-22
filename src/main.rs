#[allow(unused_imports)]
use std::io::{self, Write};

use args::parse_args;
use command::Cmd;

mod command;
mod utils;
mod args;

fn main() {
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let args = parse_args(input.trim().to_string());
    let args = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

    match Cmd::from(args[0]) {
      Cmd::Unknown => {
        println!("{}: command not found", input.trim());
      }
      command => {
        command.exec(args);
        continue;
      }
    }
  }
}
