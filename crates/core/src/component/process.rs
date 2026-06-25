use std::io::Write;
use std::process::{Command, Stdio};

use tgraphy_types::ReporterOutput as ProcessReporterOutput;

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
    fn evaluate(&self, input: EvaluatorInput) -> ComponentResult<FindingLayer> {
        let input_json =
            serde_json::to_string(&input).map_err(|e| ComponentError::InternalError {
                message: format!("failed to serialize evaluator input: {e}"),
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
                    "failed to run evaluator process '{}': {e}",
                    self.config.command
                ),
            })?;

        if !output.status.success() {
            return Err(ComponentError::InternalError {
                message: format!(
                    "evaluator process '{}' exited with status {}",
                    self.config.command, output.status
                ),
            });
        }

        serde_json::from_slice(&output.stdout).map_err(|e| ComponentError::InternalError {
            message: format!("failed to deserialize evaluator output: {e}"),
        })
    }
}

pub struct ProcessReporter {
    pub name: String,
    pub config: ProcessConfig,
}

impl Reporter for ProcessReporter {
    fn report(&self, input: ReporterInput) -> ComponentResult<ReportOutput> {
        let input_json =
            serde_json::to_string(&input).map_err(|e| ComponentError::InternalError {
                message: format!("failed to serialize reporter input: {e}"),
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
                    "failed to run reporter process '{}': {e}",
                    self.config.command
                ),
            })?;

        if !output.status.success() {
            return Err(ComponentError::InternalError {
                message: format!(
                    "reporter process '{}' exited with status {}",
                    self.config.command, output.status
                ),
            });
        }

        let envelope: ProcessReporterOutput =
            serde_json::from_slice(&output.stdout).map_err(|e| ComponentError::InvalidOutput {
                message: format!(
                    "reporter process '{}' produced invalid output envelope: {e}",
                    self.config.command
                ),
            })?;

        Ok(ReportOutput {
            format: self.name.clone(),
            extension: envelope.extension,
            content: envelope.content.into_bytes(),
        })
    }
}
