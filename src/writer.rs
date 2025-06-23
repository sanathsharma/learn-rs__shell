use std::{fs::OpenOptions, io::Write};

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
  pub fn output(self, buf: &[u8]) {
    match self.redirection {
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
      _ => {
        println!("{}", String::from_utf8_lossy(buf));
      }
    }
  }

  pub fn output_string<T: AsRef<str>>(self, string: T) {
    let string = string.as_ref();
    match self.redirection {
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
      _ => {
        println!("{}", string);
      }
    }
  }
  pub fn output_error(self, buf: &[u8]) {
    match self.redirection {
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
        eprintln!("{}", String::from_utf8_lossy(buf));
      }
    }
  }

  pub fn output_error_string<T: AsRef<str>>(self, string: T) {
    let string = string.as_ref();
    match self.redirection {
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
