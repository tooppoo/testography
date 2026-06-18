use std::collections::HashMap;

use super::evaluator::Evaluator;
use super::parser::Parser;
use super::reporter::Reporter;
use super::{ComponentError, ComponentResult};

pub struct ComponentRegistry {
    parsers: HashMap<String, Box<dyn Parser>>,
    evaluators: HashMap<String, Box<dyn Evaluator>>,
    reporters: HashMap<String, Box<dyn Reporter>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            evaluators: HashMap::new(),
            reporters: HashMap::new(),
        }
    }

    pub fn register_parser(&mut self, name: impl Into<String>, parser: Box<dyn Parser>) {
        self.parsers.insert(name.into(), parser);
    }

    pub fn register_evaluator(&mut self, name: impl Into<String>, evaluator: Box<dyn Evaluator>) {
        self.evaluators.insert(name.into(), evaluator);
    }

    pub fn register_reporter(&mut self, name: impl Into<String>, reporter: Box<dyn Reporter>) {
        self.reporters.insert(name.into(), reporter);
    }

    pub fn resolve_parser(&self, name: &str) -> ComponentResult<&dyn Parser> {
        self.parsers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| ComponentError::UnsupportedComponent {
                message: format!("no parser registered with name {name:?}"),
            })
    }

    pub fn resolve_evaluator(&self, name: &str) -> ComponentResult<&dyn Evaluator> {
        self.evaluators
            .get(name)
            .map(|e| e.as_ref())
            .ok_or_else(|| ComponentError::UnsupportedComponent {
                message: format!("no evaluator registered with name {name:?}"),
            })
    }

    pub fn resolve_reporter(&self, name: &str) -> ComponentResult<&dyn Reporter> {
        self.reporters
            .get(name)
            .map(|r| r.as_ref())
            .ok_or_else(|| ComponentError::UnsupportedComponent {
                message: format!("no reporter registered with name {name:?}"),
            })
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
