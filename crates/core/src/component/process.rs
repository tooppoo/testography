use crate::artifact::ParsedEvidenceArtifact;
use crate::artifact::staged::FindingLayer;

use super::evaluator::{Evaluator, EvaluatorInput};
use super::parser::{Parser, ParserInput};
use super::reporter::{ReportOutput, Reporter, ReporterInput};
use super::{ComponentError, ComponentResult};

pub struct ProcessConfig {
    pub command: String,
    pub args: Vec<String>,
}

pub struct ProcessParser {
    pub config: ProcessConfig,
}

impl Parser for ProcessParser {
    fn parse(&self, _input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact> {
        Err(ComponentError::InternalError {
            message: "process-based parser execution is not yet implemented".to_string(),
        })
    }
}

pub struct ProcessEvaluator {
    pub config: ProcessConfig,
}

impl Evaluator for ProcessEvaluator {
    fn evaluate(&self, _input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        Err(ComponentError::InternalError {
            message: "process-based evaluator execution is not yet implemented".to_string(),
        })
    }
}

pub struct ProcessReporter {
    pub config: ProcessConfig,
}

impl Reporter for ProcessReporter {
    fn report(&self, _input: ReporterInput) -> ComponentResult<ReportOutput> {
        Err(ComponentError::InternalError {
            message: "process-based reporter execution is not yet implemented".to_string(),
        })
    }
}
