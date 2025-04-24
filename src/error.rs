// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum XplError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parse error: {0}")]
    Xml(#[from] xmltree::ParseError),

    #[error("Semantic error: {0}")]
    Semantic(String),
}
