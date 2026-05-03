use thiserror::Error;

use crate::ast::Span;

/// All errors that can be produced at any stage of the pipeline.
#[derive(Debug, Error)]
pub enum YolangError {
    #[error("Parse error in {filename} at {start}..{end}: {message}")]
    ParseError { message: String, start: usize, end: usize, filename: String },

    #[error("Parse error in {filename} at {start}..{end}, line {line}: {message}")]
    ParseErrorWithLine { message: String, start: usize, end: usize, line: String, filename: String },

    #[error("Type error in {filename} at {start}..{end}: {message}")]
    TypeError { message: String, start: usize, end: usize, filename: String },

    #[error("Panic at {filename} {start}..{end}: {message}")]
    RuntimePanic { message: String, start: usize, end: usize, filename: String },

    /// A parser invariant was violated — this indicates a bug in the parser, not
    /// invalid user input. The message names the function and what was expected.
    #[error("internal parser error: {message}")]
    Internal { message: String },
}

impl YolangError {
    pub fn parse(msg: impl Into<String>, span: &Span) -> Self {
        Self::ParseError {
            message: msg.into(),
            start: span.start,
            end: span.end,
            filename: span.filename.clone(),
        }
    }

    pub fn type_error(msg: impl Into<String>, span: &Span) -> Self {
        Self::TypeError {
            message: msg.into(),
            start: span.start,
            end: span.end,
            filename: span.filename.clone(),
        }
    }

    pub fn panic(msg: impl Into<String>, span: &Span) -> Self {
        Self::RuntimePanic {
            message: msg.into(),
            start: span.start,
            end: span.end,
            filename: span.filename.clone(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal { message: msg.into() }
    }
}
