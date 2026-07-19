use std::collections::HashMap;

use crate::{compiler::compiler::CompileState, project::config::Project};

pub mod decl;
pub mod expr;
pub mod stmt;
pub mod weave_analyser;

pub struct WeaveAnalyzerContext {
    // The path of the source file being analyzed
    pub source_path: String,

    // Project config
    pub project: Option<Project>,

    // Map of all compiled imported scrolls
    pub tethered_scrolls: HashMap<String, CompileState>,

    // This flag indicates whether the analyzer is running in import mode or not.
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
