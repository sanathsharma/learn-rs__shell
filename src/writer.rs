use std::io::{BufReader, Read};
use std::process::Child;
use std::sync::Arc;
use std::{fs::OpenOptions, io::{Seek, SeekFrom, Write}, thread};

#[derive(Debug)]
pub enum CmdOutput {
  Stdout(String),
  Stderr(String),
  StdoutBytes(Vec<u8>),
  StderrBytes(Vec<u8>),
  Stream(Child),
}

#[derive(Debug, Clone)]
pub enum Redirection {
  // Redirect stdout output into a file
  Stdout { file_path: String, append: bool },
  // Redirect stderr output into a file
  Stderr { file_path: String, append: bool },
  // print to terminal console
  None,
}

#[derive(Debug, Clone)]
pub struct CmdOutputWriter {
  redirection: Redirection,
}

impl CmdOutputWriter {
  pub fn new(redirection: Redirection) -> Self {
    Self { redirection }
  }
}

impl CmdOutputWriter {
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
      Redirection::Stdout { file_path, append } => {
        let file = OpenOptions::new()
          .write(true)
          .append(append)
          .create(true)
          .open(&file_path);

        let write = match file {
          Ok(mut file) => {
            let file_size = file.seek(SeekFrom::End(0)).unwrap();
            if append && file_size > 0 {
              write!(file, "\n{}", String::from_utf8_lossy(buf))
            } else {
              file.write_all(buf)
            }
          }
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
      Redirection::Stderr { file_path, append } => {
        print_to_stdout();
        let _ = OpenOptions::new()
          .create(true)
          .append(append)
          .write(true)
          .open(&file_path);
      }
      Redirection::None => print_to_stdout(),
    }
  }

  pub fn output_string<T: AsRef<str>>(&self, string: T) {
    let string = string.as_ref();
    match self.redirection.clone() {
      Redirection::Stdout { file_path, append } => {
        let file = OpenOptions::new()
          .write(true)
          .append(append)
          .create(true)
          .open(&file_path);

        let write = match file {
          Ok(mut file) => {
            let file_size = file.seek(SeekFrom::End(0)).unwrap();
            if append && file_size > 0 {
              write!(file, "\n{}", string)
            } else {
              write!(file, "{}", string)
            }
          }
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
      Redirection::Stderr { file_path, append } => {
        println!("{}", string);
        let _ = OpenOptions::new()
          .create(true)
          .append(append)
          .write(true)
          .open(&file_path);
      }
      Redirection::None => {
        println!("{}", string);
      }
    }
  }
  pub fn output_error(&self, buf: &[u8]) {
    let print_to_stderr = || {
      if buf.is_empty() {
        return;
      }
      if buf.ends_with(b"\n") {
        eprint!("{}", String::from_utf8_lossy(buf));
        return;
      }
      eprintln!("{}", String::from_utf8_lossy(buf));
    };

    match self.redirection.clone() {
      Redirection::Stderr { file_path, append } => {
        let file = OpenOptions::new()
          .write(true)
          .append(append)
          .create(true)
          .open(&file_path);

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
      Redirection::Stdout { file_path, append } => {
        print_to_stderr();
        let _ = OpenOptions::new()
          .create(true)
          .append(append)
          .write(true)
          .open(&file_path);
      }
      Redirection::None => {}
    }
  }

  pub fn output_error_string<T: AsRef<str>>(&self, string: T) {
    let string = string.as_ref();
    match self.redirection.clone() {
      Redirection::Stderr { file_path, append } => {
        let file = OpenOptions::new()
          .write(true)
          .append(append)
          .create(true)
          .open(&file_path);

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
      Redirection::Stdout { file_path, append } => {
        eprintln!("{}", string);
        let _ = OpenOptions::new()
          .create(true)
          .append(append)
          .write(true)
          .open(&file_path);
      }
      Redirection::None => {
        eprintln!("{}", string);
      }
    }
  }

  pub fn write_cmd_output(&self, cmd_output: CmdOutput) {
    match cmd_output {
      CmdOutput::Stdout(string) => self.output_string(string),
      CmdOutput::StdoutBytes(bytes) => self.output(&bytes),
      CmdOutput::Stderr(string) => self.output_error_string(string),
      CmdOutput::StderrBytes(bytes) => self.output(&bytes),
      CmdOutput::Stream(mut child) => {
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let writer = Arc::new(self.clone());
        let writer_stdout = Arc::clone(&writer);
        let writer_stderr = Arc::clone(&writer);

        let _stdout_handle = thread::spawn(move || {
          let mut reader = BufReader::new(stdout);
          let mut buf = [0u8; 4096];
          loop {
            let size = reader.read(&mut buf).unwrap();
            if size == 0 {
              continue;
            }
            writer_stdout.output(&buf[..size]);
          }
        });

        let _stderr_handle = thread::spawn(move || {
          let mut reader = BufReader::new(stderr);
          let mut buf = [0u8; 4096];
          loop {
            let size = reader.read(&mut buf).unwrap();
            if size == 0 {
              continue;
            }
            writer_stderr.output_error(&buf[..size]);
          }
        });

        child.wait().unwrap();
        // NOTE: don't join the thread handles, as we don't want to wait for thread to complete when program already has
        // stdout_handle.join().unwrap();
        // stderr_handle.join().unwrap();
      }
    }
  }
}

// TODO
// - [ ] kill waiting child process on ctrl-c