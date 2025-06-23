use std::{
  fs::{File, OpenOptions},
  io::Write,
};

#[derive(Debug, Clone)]
pub enum Redirection {
  // Redirect stdout output into a file
  Stdout { file_path: String },
  // Redirect stderr output into a file
  Stderr { file_path: String },
  // Redirect stdout & stderr output into a file
  Any { file_path: String },
  // print to terminal console
  None,
}

#[derive(Debug, Clone)]
pub struct CmdOuputWriter {
  redirection: Redirection,
}

impl CmdOuputWriter {
  pub fn new(redirection: Redirection) -> Self {
    Self { redirection }
  }
}

impl CmdOuputWriter {
  pub fn output(&self, buf: &[u8]) {
    let print_to_stdout = || {
      if buf.is_empty() {
        return;
      }
      if buf.ends_with(b"\n") {
        print!("{}", String::from_utf8_lossy(buf));
        return;
      }
      println!("{}", String::from_utf8_lossy(buf));
    };

    match self.redirection.clone() {
      Redirection::Stdout { file_path } | Redirection::Any { file_path } => {
        let file = OpenOptions::new().write(true).create(true).open(&file_path);
        let write = match file {
          Ok(mut file) => file.write_all(buf),
          Err(_) => {
            eprintln!("Error opening file {}", file_path);
            return;
          }
        };

        match write {
          Ok(_) => {}
          Err(_) => eprintln!("Error writing to {}", file_path),
        }
      }
      Redirection::Stderr { file_path } => {
        print_to_stdout();
        let _ = OpenOptions::new().create(true).write(true).open(&file_path);
      }
      Redirection::None => print_to_stdout(),
    }
  }

  pub fn output_string<T: AsRef<str>>(&self, string: T) {
    let string = string.as_ref();
    match self.redirection.clone() {
      Redirection::Stdout { file_path } | Redirection::Any { file_path } => {
        let file = OpenOptions::new().write(true).create(true).open(&file_path);
        let write = match file {
          Ok(mut file) => file.write_all(string.as_bytes()),
          Err(_) => {
            eprintln!("Error opening file {}", file_path);
            return;
          }
        };

        match write {
          Ok(_) => {}
          Err(_) => eprintln!("Error writing to {}", file_path),
        }
      }
      Redirection::Stderr { file_path } => {
        println!("{}", string);
        let _ = OpenOptions::new().create(true).write(true).open(&file_path);
      }
      Redirection::None => {
        println!("{}", string);
      }
    }
  }
  pub fn output_error(&self, buf: &[u8]) {
    match self.redirection.clone() {
      Redirection::Stderr { file_path } | Redirection::Any { file_path } => {
        let file = OpenOptions::new().write(true).create(true).open(&file_path);
        let write = match file {
          Ok(mut file) => file.write_all(buf),
          Err(_) => {
            eprintln!("Error opening file {}", file_path);
            return;
          }
        };

        match write {
          Ok(_) => {}
          Err(_) => eprintln!("Error writing to {}", file_path),
        }
      }
      _ => {
        if buf.is_empty() {
          return;
        }
        if buf.ends_with(b"\n") {
          eprint!("{}", String::from_utf8_lossy(buf));
          return;
        }
        eprintln!("{}", String::from_utf8_lossy(buf));
      }
    }
  }

  pub fn output_error_string<T: AsRef<str>>(&self, string: T) {
    let string = string.as_ref();
    match self.redirection.clone() {
      Redirection::Stderr { file_path } | Redirection::Any { file_path } => {
        let file = OpenOptions::new().write(true).create(true).open(&file_path);
        let write = match file {
          Ok(mut file) => file.write_all(string.as_bytes()),
          Err(_) => {
            eprintln!("Error opening file {}", file_path);
            return;
          }
        };

        match write {
          Ok(_) => {}
          Err(_) => eprintln!("Error writing to {}", file_path),
        }
      }
      _ => {
        eprintln!("{}", string);
      }
    }
  }
}
