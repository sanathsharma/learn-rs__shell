#[derive(Debug, Clone, Copy)]
pub enum AnsiCode {
  /// Clear entire screen
  ClearScreen,
  /// Move cursor to top-left corner (position 1,1)
  CursorHome,
  /// Clear from cursor position to the end of the current line
  ClearToEndOfLine,
  /// Clear from the beginning of current line to cursor position
  ClearToStartOfLine,
  /// Clear the entire current line
  ClearEntireLine,
  /// Clear from cursor position to the end of screen
  ClearToEndOfScreen,
  /// Clear from cursor position to the beginning of screen
  ClearToStartOfScreen,
  /// Move the cursor to the beginning of the current line (carriage return)
  CarriageReturn,
  CRLF,
  MoveCursorLeft,
  MoveCursorRight,
  BEL,
}

impl AnsiCode {
  /// Get the ANSI escape sequence as a string
  pub fn as_str(&self) -> &'static str {
    match self {
      AnsiCode::ClearScreen => "\x1b[2J",
      AnsiCode::CursorHome => "\x1b[H",
      AnsiCode::ClearToEndOfLine => "\x1b[K",
      AnsiCode::ClearToStartOfLine => "\x1b[1K",
      AnsiCode::ClearEntireLine => "\x1b[2K",
      AnsiCode::ClearToEndOfScreen => "\x1b[J", // or \x1b[0J
      AnsiCode::ClearToStartOfScreen => "\x1b[1J",
      AnsiCode::CarriageReturn => "\r",
      AnsiCode::MoveCursorLeft => "\x1b[D",
      AnsiCode::MoveCursorRight => "\x1b[C",
      AnsiCode::CRLF => "\r\n",
      AnsiCode::BEL => "\x07",
    }
  }

  /// Get the ANSI escape sequence as bytes
  pub fn as_bytes(&self) -> &'static [u8] {
    self.as_str().as_bytes()
  }

  pub fn write(&self) -> () {
    print!("{}", self.as_str());
  }
}

impl std::fmt::Display for AnsiCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
