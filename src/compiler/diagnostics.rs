use std::path::PathBuf;

pub struct Diagnostic {
    pub message: String,
    pub location: SourceLocation,
    pub phase: CompilationPhase,
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
}

//. Diagnostic collector that can accumulate errors (curses) and warnings (omens) during compilation phases.
// why not a bit of thematicity
pub struct Augury {
    pub curses: Vec<Diagnostic>,
    pub omens: Vec<Diagnostic>,
}

impl Augury {
    pub fn new() -> Self {
        Self {
            curses: Vec::new(),
            omens: Vec::new(),
        }
    }

    pub fn is_cursed(&self) -> bool {
        !self.curses.is_empty()
    }

    pub fn forsee_curse(
        &mut self,
        source: SourceLocation,
        message: String,
        phase: CompilationPhase,
    ) {
        let diagnostic = Diagnostic {
            message,
            location: source,
            phase,
        };
        self.curses.push(diagnostic);
    }

    pub fn forsee_omen(
        &mut self,
        source: SourceLocation,
        message: String,
        phase: CompilationPhase,
    ) {
        let diagnostic = Diagnostic {
            message,
            location: source,
            phase,
        };
        self.omens.push(diagnostic);
    }

    pub fn list_curses(&self) -> Vec<String> {
        self.curses
            .iter()
            .map(|c| {
                format!(
                    "curse: {}\nat [\x1b[4m{}:{}:{}\x1b[0m]\n",
                    c.message,
                    c.location.file.display(),
                    c.location.line,
                    c.location.column,
                )
            })
            .collect()
    }
}
