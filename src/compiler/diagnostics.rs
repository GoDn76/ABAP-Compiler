use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn zero() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticBag {
    pub items: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>, start: Position, end: Position) {
        self.items.push(Diagnostic {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            start,
            end,
        });
    }

    pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>, start: Position, end: Position) {
        self.items.push(Diagnostic {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            start,
            end,
        });
    }

    pub fn info(&mut self, code: impl Into<String>, message: impl Into<String>, start: Position, end: Position) {
        self.items.push(Diagnostic {
            severity: Severity::Info,
            code: code.into(),
            message: message.into(),
            start,
            end,
        });
    }

    pub fn extend(&mut self, other: DiagnosticBag) {
        self.items.extend(other.items);
    }
}

pub fn format_diagnostic(d: &Diagnostic) -> String {
    let sev = match d.severity {
        Severity::Error => "ERROR",
        Severity::Warning => "WARN",
        Severity::Info => "INFO",
    };
    format!("{}:{} {} {}: {}", d.start.line, d.start.column, sev, d.code, d.message)
}
