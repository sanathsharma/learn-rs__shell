#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parts = input.trim().split_whitespace().collect::<Vec<&str>>();

        // When no code is supplied
        if input.trim().starts_with("exit") && parts.len() == 1 {
            process::exit(255);
        }

        // When a valid code is supplied
        if input.trim().starts_with("exit") && parts.len() == 2 {
            let code = parts[1].parse::<u8>().expect("exit: invalid code");
            process::exit(code.into());
        }

        println!("{}: command not found", input.trim());
    }
}
