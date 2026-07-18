use std::collections::HashMap;

use crate::{compiler::compiler::CompileState, project::config::Project};

pub mod weave_analyser;
pub mod decl;
pub mod expr;
pub mod stmt;

pub struct WeaveAnalyzerContext {
    pub source_path: String,
    pub project: Option<Project>,
    pub tethered_scrolls: HashMap<String, CompileState>,
    pub import_mode: bool,
}

impl WeaveAnalyzerContext {
    pub fn new(source_path: String, project: Option<Project>, import_mode: bool) -> Self {
        WeaveAnalyzerContext {
            source_path,
            project,
            import_mode,
            tethered_scrolls: HashMap::new(),
        }
    }
}