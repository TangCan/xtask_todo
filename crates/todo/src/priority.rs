//! Priority for a todo item.

use std::fmt;
use std::str::FromStr;

/// Priority for a todo item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }
}

impl FromStr for Priority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "low" | "1" => Ok(Self::Low),
            "medium" | "2" => Ok(Self::Medium),
            "high" | "3" => Ok(Self::High),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
        }
    }
}
