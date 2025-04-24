// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum XplError {
    #[error("IO error in {file}: {source}")]
    Io {
        source: std::io::Error,
        file: String,
    },

    #[error("XML parse error in {file}: {source}")]
    Xml {
        source: xmltree::ParseError,
        file: String,
    },

    #[error("{file}:{line}:{col}: {msg}")]
    Semantic {
        msg: String,
        file: String,
        line: usize,
        col: usize,
    },
}

impl XplError {
    /// Print the error with colors and source arrow
    pub fn pretty_print(&self) {
        use ansi_term::Colour::{Blue, Red, Yellow};
        match self {
            XplError::Io { source, file } => {
                eprintln!("{}: {} in file {}", Red.bold().paint("error"), source, file);
            }
            XplError::Xml { source, file } => {
                eprintln!("{}: {} in file {}", Red.bold().paint("error"), source, file);
            }
            XplError::Semantic {
                msg,
                file,
                line,
                col,
            } => {
                // header
                eprintln!("{}: {}", Red.bold().paint("error"), Yellow.paint(msg));
                // location
                eprintln!("  {} {}:{}:{}", Blue.paint("-->"), file, line, col);
                // source context
                if let Ok(src) = std::fs::read_to_string(file) {
                    if let Some(src_line) = src.lines().nth(*line - 1) {
                        // blank gutter line
                        eprintln!("  {}", Blue.paint("|"));
                        // code line without number
                        eprintln!("  {} {}", Blue.paint("|"), src_line);
                        // arrow line (align caret under code)
                        let indent = " ".repeat(col.saturating_sub(1));
                        eprintln!("  {} {}{}", Blue.paint("|"), indent, Red.paint("^"));
                    }
                }
            }
        }
    }
}
