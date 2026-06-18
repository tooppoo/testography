pub mod builtin;
pub mod evaluator;
pub mod parser;
pub mod process;
pub mod registry;
pub mod reporter;

pub use evaluator::{Evaluator, EvaluatorInput};
pub use parser::{Parser, ParserInput};
pub use registry::ComponentRegistry;
pub use reporter::{ReportOutput, Reporter, ReporterInput};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Parser,
    Evaluator,
    Reporter,
}

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("component execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("component produced invalid output: {message}")]
    InvalidOutput { message: String },

    #[error("unsupported component: {message}")]
    UnsupportedComponent { message: String },

    #[error("component internal error: {message}")]
    InternalError { message: String },
}

pub type ComponentResult<T> = Result<T, ComponentError>;
