// Bash impl docs, see https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output

#[allow(unused_imports)]
use std::io::{self, Write};

use args::parse_args;
use command::Cmd;

mod args;
mod command;
mod utils;
mod writer;

fn main() {
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let cmd_args = parse_args(input.trim().to_string());
    // println!(">>> args: {:?}", cmd_args);

    match Cmd::from(cmd_args.args[0].clone()) {
      Cmd::Unknown => {
        println!("{}: command not found", input.trim());
      }
      command => {
        command.exec(cmd_args);
        continue;
      }
    }
  }
}
