use thiserror::Error;

use crate::ast::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    E0001, // Type mismatch
    E0002, // Annotation required
    E0003, // Undefined name
    E0004, // Arity mismatch
    E0005, // Invalid operand types
    E0006, // Assignment to immutable binding
    E0007, // Invalid cast
    E0008, // Non-exhaustive match
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::E0001 => write!(f, "E0001"),
            ErrorCode::E0002 => write!(f, "E0002"),
            ErrorCode::E0003 => write!(f, "E0003"),
            ErrorCode::E0004 => write!(f, "E0004"),
            ErrorCode::E0005 => write!(f, "E0005"),
            ErrorCode::E0006 => write!(f, "E0006"),
            ErrorCode::E0007 => write!(f, "E0007"),
            ErrorCode::E0008 => write!(f, "E0008"),
        }
    }
}

/// All errors that can be produced at any stage of the pipeline.
#[derive(Debug, Error)]
pub enum YoloscriptError {
    #[error("Parse error in {filename} at {start}..{end}: {message}")]
    ParseError { message: String, start: usize, end: usize, filename: String },

    #[error("Parse error in {filename} at {start}..{end}, line {line}: {message}")]
    ParseErrorWithLine { message: String, start: usize, end: usize, line: String, filename: String },

    #[error("[{code}] type error in {filename} at {start}..{end}: {message}")]
    TypeError { code: ErrorCode, message: String, start: usize, end: usize, filename: String },

    #[error("Panic at {filename} {start}..{end}: {message}")]
    RuntimePanic { message: String, start: usize, end: usize, filename: String },

    /// A parser invariant was violated — this indicates a bug in the parser, not
    /// invalid user input. The message names the function and what was expected.
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl YoloscriptError {
    pub fn parse(msg: impl Into<String>, span: &Span) -> Self {
        Self::ParseError {
            message: msg.into(),
            start: span.start,
            end: span.end,
            filename: span.filename.clone(),
        }
    }

    pub fn type_error(code: ErrorCode, msg: impl Into<String>, span: &Span) -> Self {
        Self::TypeError {
            code,
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
