use std::{fmt, io, io::IsTerminal, str::FromStr};

use clap::ValueEnum;
use thiserror::Error;

/// Whether messages should use color output.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorChoice {
    /// Force color output.
    Always,
    /// Intelligently guess whether to use color output.
    Auto,
    /// Force disable color output.
    Never,
}

impl fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => f.write_str("always"),
            Self::Auto => f.write_str("auto"),
            Self::Never => f.write_str("never"),
        }
    }
}

#[derive(Debug, Error)]
#[error("expected `always` or `never`, got `{}`", self.0)]
pub struct ParseColorChoiceError(String);

impl FromStr for ColorChoice {
    type Err = ParseColorChoiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" => Ok(Self::Always),
            "auto" => Ok(Self::Auto),
            "never" => Ok(Self::Never),
            s => Err(ParseColorChoiceError(s.to_string())),
        }
    }
}

impl ColorChoice {
    pub fn is_color(self) -> bool {
        match self {
            Self::Always => true,
            Self::Auto => io::stderr().is_terminal(),
            Self::Never => false,
        }
    }
}
