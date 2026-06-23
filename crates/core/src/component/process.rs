use std::io::Write;
use std::process::{Command, Stdio};

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
    fn parse(&self, input: ParserInput) -> ComponentResult<ParsedEvidenceArtifact> {
        let input_json =
            serde_json::to_string(&input).map_err(|e| ComponentError::InternalError {
                message: format!("failed to serialize parser input: {e}"),
            })?;

        let output = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .and_then(|mut child| {
                child
                    .stdin
                    .take()
                    .expect("stdin was piped")
                    .write_all(input_json.as_bytes())?;
                child.wait_with_output()
            })
            .map_err(|e| ComponentError::InternalError {
                message: format!(
                    "failed to run parser process '{}': {e}",
                    self.config.command
                ),
            })?;

        if !output.status.success() {
            return Err(ComponentError::InternalError {
                message: format!(
                    "parser process '{}' exited with status {}",
                    self.config.command, output.status
                ),
            });
        }

        serde_json::from_slice(&output.stdout).map_err(|e| ComponentError::InternalError {
            message: format!("failed to deserialize parser output: {e}"),
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
