use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlantUmlError {
    #[error("failed to extract plantuml binary: {0}")]
    BinaryExtraction(#[source] std::io::Error),

    #[error("plantuml process failed with exit code {code}: {stderr}")]
    ProcessFailed { code: i32, stderr: String },

    #[error("plantuml process terminated by signal")]
    ProcessSignaled,

    #[error("failed to spawn plantuml process: {0}")]
    ProcessSpawn(#[source] std::io::Error),

    #[error("failed to write to plantuml stdin: {0}")]
    StdinWrite(#[source] std::io::Error),

    #[error("failed to read input file '{path}': {source}")]
    InputRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write output file '{path}': {source}")]
    OutputWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid UTF-8 in plantuml output: {0}")]
    InvalidUtf8(#[source] std::string::FromUtf8Error),

    #[error("plantuml syntax error: {0}")]
    SyntaxError(String),
}

pub type Result<T> = std::result::Result<T, PlantUmlError>;
