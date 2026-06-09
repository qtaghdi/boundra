use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCode {
    Br001,
    Br002,
    Br003,
    Br004,
}

impl RuleCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Br001 => "BR-001",
            Self::Br002 => "BR-002",
            Self::Br003 => "BR-003",
            Self::Br004 => "BR-004",
        }
    }
}

impl Display for RuleCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: RuleCode,
    pub file: String,
    pub line: usize,
    pub import_path: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Client,
    Server,
    Shared,
    Mcp,
    Tests,
    Unknown,
}
