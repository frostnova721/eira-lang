use std::fmt::{self, Display};

/// Where in the source a runtime error happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl Display for RuntimeLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// One active call at the moment of the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceFrame {
    /// The spell's name, or `None` for top-level scroll code.
    pub spell: Option<String>,
    pub location: Option<RuntimeLocation>,
}

/// An error raised while running Eira bytecode.
///
/// Contains only owned, thread-safe data, so a host can move it anywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
    /// Active calls, innermost first. Empty for errors raised before any code ran
    /// (e.g. calling an unknown spell through [crate::EiraVM::cast]).
    pub trace: Vec<TraceFrame>,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        RuntimeError {
            message: message.into(),
            trace: vec![],
        }
    }

    /// Location of the instruction that failed, if known.
    pub fn location(&self) -> Option<&RuntimeLocation> {
        self.trace.first().and_then(|f| f.location.as_ref())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        // Collapse runs of identical frames (deep recursion) into one line.
        let mut i = 0;
        while i < self.trace.len() {
            let frame = &self.trace[i];
            let mut repeats = 0;
            while i + repeats + 1 < self.trace.len() && self.trace[i + repeats + 1] == *frame {
                repeats += 1;
            }

            let place = match &frame.spell {
                Some(name) => format!("spell '{}'", name),
                None => "top level".to_string(),
            };
            match &frame.location {
                Some(loc) => write!(f, "\n  at {} ({})", loc, place)?,
                None => write!(f, "\n  in {}", place)?,
            }
            if repeats > 0 {
                write!(f, "\n  ... repeated {} more times", repeats)?;
            }
            i += repeats + 1;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}
