use std::path::PathBuf;

use crate::compiler::scroll_reader::ScrollReader;

pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
    pub phase: CompilationPhase,
}

pub enum Severity {
    Error,
    Warning,
    Info,
}

pub enum CompilationPhase {
    Scan,
    Parse,
    Weave,
    CodeGen,
}

pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: Option<usize>,
}

pub struct DiagnosticEngine {
    pub diagnostics: Vec<Diagnostic>,
    pub scroll_reader: ScrollReader,
}

impl DiagnosticEngine {
    
}