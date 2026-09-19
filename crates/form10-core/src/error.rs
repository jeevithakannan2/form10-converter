use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateField {
    OldMember,
    OldSociety,
    OldUnion,
    NewMember,
    NewSociety,
    NewUnion,
}

impl RateField {
    pub fn label(self) -> &'static str {
        match self {
            Self::OldMember => "old member rate",
            Self::OldSociety => "old society rate",
            Self::OldUnion => "old union rate",
            Self::NewMember => "new member rate",
            Self::NewSociety => "new society rate",
            Self::NewUnion => "new union rate",
        }
    }
}

impl Display for RateField {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateValidationIssue {
    NotANumber,
    Negative,
}

impl Display for RateValidationIssue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotANumber => "must be a valid number",
            Self::Negative => "cannot be negative",
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    CouldNotOpenWorkbook { message: String },
    CouldNotReadSheet { sheet_name: String, message: String },
    ProcurementTableNotFound,
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CouldNotOpenWorkbook { message } => {
                write!(formatter, "Could not open workbook: {message}")
            }
            Self::CouldNotReadSheet {
                sheet_name,
                message,
            } => write!(formatter, "Could not read sheet '{sheet_name}': {message}"),
            Self::ProcurementTableNotFound => formatter.write_str(
                "Could not find a month-wise procurement table. Expected headers for M.No, Member Name, and at least one month from APR through MAR.",
            ),
        }
    }
}

impl Error for ParseError {}

#[derive(Debug)]
pub enum ImportError {
    UnsupportedExtension,
    Parse(ParseError),
}

impl Display for ImportError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedExtension => formatter.write_str("Choose an .xls or .xlsx file."),
            Self::Parse(error) => error.fmt(formatter),
        }
    }
}

impl Error for ImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedExtension => None,
            Self::Parse(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsValidationError {
    InvalidFinancialYear,
    InvalidRate {
        field: RateField,
        issue: RateValidationIssue,
    },
    InvalidNewFromMonth(u8),
}

impl Display for SettingsValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFinancialYear => formatter.write_str("Use a year like 2025-26."),
            Self::InvalidRate { field, issue } => match issue {
                RateValidationIssue::NotANumber => write!(formatter, "Check {field}."),
                RateValidationIssue::Negative => write!(formatter, "{field} cannot be negative."),
            },
            Self::InvalidNewFromMonth(month) => {
                write!(
                    formatter,
                    "New rate start month must be between 0 and 12, found {month}."
                )
            }
        }
    }
}

impl Error for SettingsValidationError {}

#[derive(Debug)]
pub enum PreviewError {
    Validation(SettingsValidationError),
    SourceWouldBeOverwritten,
}

impl Display for PreviewError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::SourceWouldBeOverwritten => {
                formatter.write_str("Choose a different name. The source file cannot be replaced.")
            }
        }
    }
}

impl Error for PreviewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(error) => Some(error),
            Self::SourceWouldBeOverwritten => None,
        }
    }
}

#[derive(Debug)]
pub enum ExportError {
    SourceWouldBeOverwritten,
    DestinationExists(PathBuf),
    WriteFailed(String),
}

impl Display for ExportError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceWouldBeOverwritten => {
                formatter.write_str("Choose a different name. The source file cannot be replaced.")
            }
            Self::DestinationExists(path) => {
                write!(formatter, "Output file already exists: {}", path.display())
            }
            Self::WriteFailed(message) => formatter.write_str(message),
        }
    }
}

impl Error for ExportError {}
