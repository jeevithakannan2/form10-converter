use std::path::{Path, PathBuf};

use crate::error::{
    ExportError, ImportError, PreviewError, RateField, RateValidationIssue, SettingsValidationError,
};
use crate::generator::generate_form10;
use crate::model::{ContributionKind, Rates, Settings, SourceData};
use crate::parser::parse_source;

#[derive(Debug, Default, Clone, Copy)]
pub struct ConverterService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSource {
    pub path: PathBuf,
    pub data: SourceData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsInput {
    pub financial_year: String,
    pub dcmpu: String,
    pub district: String,
    pub society: String,
    pub society_code: String,
    pub old_member: String,
    pub old_society: String,
    pub old_union: String,
    pub new_member: String,
    pub new_society: String,
    pub new_union: String,
    pub new_from_month: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionSummary {
    pub member_count: usize,
    pub active_subscriptions: usize,
    pub member_contribution: f64,
    pub society_contribution: f64,
    pub union_contribution: f64,
    pub settings: Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportPreview {
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub destination_exists: bool,
    pub suggested_file_name: String,
    pub member_count: usize,
    pub active_subscriptions: usize,
    pub member_contribution: f64,
    pub society_contribution: f64,
    pub union_contribution: f64,
    pub settings: Settings,
}

impl ConverterService {
    pub fn new() -> Self {
        Self
    }

    pub fn import(&self, path: impl AsRef<Path>) -> Result<ImportedSource, ImportError> {
        let path = path.as_ref();
        if !is_excel_file(path) {
            return Err(ImportError::UnsupportedExtension);
        }

        Ok(ImportedSource {
            path: path.to_path_buf(),
            data: parse_source(path).map_err(ImportError::Parse)?,
        })
    }

    pub fn summarize(
        &self,
        source: &SourceData,
        input: SettingsInput,
    ) -> Result<ConversionSummary, SettingsValidationError> {
        let settings = validate_settings(input)?;
        let active_subscriptions = source
            .members
            .iter()
            .map(|member| {
                source
                    .active_months_for(member)
                    .iter()
                    .filter(|active| **active)
                    .count()
            })
            .sum();
        let contribution_total = |kind| {
            source
                .members
                .iter()
                .map(|member| {
                    settings
                        .rates
                        .contribution(&source.active_months_for(member), kind)
                })
                .sum()
        };

        Ok(ConversionSummary {
            member_count: source.members.len(),
            active_subscriptions,
            member_contribution: contribution_total(ContributionKind::Member),
            society_contribution: contribution_total(ContributionKind::Society),
            union_contribution: contribution_total(ContributionKind::Union),
            settings,
        })
    }

    pub fn preview(
        &self,
        source: &ImportedSource,
        input: SettingsInput,
        destination: impl AsRef<Path>,
    ) -> Result<ExportPreview, PreviewError> {
        let summary = self
            .summarize(&source.data, input)
            .map_err(PreviewError::Validation)?;
        let destination_path = ensure_xlsx_extension(destination.as_ref().to_path_buf());

        if paths_match(&source.path, &destination_path) {
            return Err(PreviewError::SourceWouldBeOverwritten);
        }

        Ok(ExportPreview {
            source_path: source.path.clone(),
            destination_exists: destination_path.exists(),
            suggested_file_name: format!("FORM-10-{}.xlsx", summary.settings.financial_year),
            member_count: summary.member_count,
            active_subscriptions: summary.active_subscriptions,
            member_contribution: summary.member_contribution,
            society_contribution: summary.society_contribution,
            union_contribution: summary.union_contribution,
            destination_path,
            settings: summary.settings,
        })
    }

    pub fn export(
        &self,
        source: &SourceData,
        preview: &ExportPreview,
        overwrite_existing: bool,
    ) -> Result<PathBuf, ExportError> {
        if paths_match(&preview.source_path, &preview.destination_path) {
            return Err(ExportError::SourceWouldBeOverwritten);
        }
        if preview.destination_path.exists() && !overwrite_existing {
            return Err(ExportError::DestinationExists(
                preview.destination_path.clone(),
            ));
        }

        generate_form10(source, &preview.settings, &preview.destination_path)
            .map_err(ExportError::WriteFailed)?;

        Ok(preview.destination_path.clone())
    }
}

fn validate_settings(input: SettingsInput) -> Result<Settings, SettingsValidationError> {
    if !valid_financial_year(&input.financial_year) {
        return Err(SettingsValidationError::InvalidFinancialYear);
    }
    if input.society.trim().is_empty() {
        return Err(SettingsValidationError::EmptySocietyName);
    }
    if input.society_code.trim().is_empty() {
        return Err(SettingsValidationError::EmptySocietyCode);
    }
    if input.new_from_month > 12 {
        return Err(SettingsValidationError::InvalidNewFromMonth(
            input.new_from_month,
        ));
    }

    Ok(Settings {
        financial_year: input.financial_year.trim().to_owned(),
        dcmpu: input.dcmpu.trim().to_owned(),
        district: input.district.trim().to_owned(),
        society: input.society.trim().to_owned(),
        society_code: input.society_code.trim().to_owned(),
        rates: Rates {
            old_member: parse_rate(RateField::OldMember, &input.old_member)?,
            old_society: parse_rate(RateField::OldSociety, &input.old_society)?,
            old_union: parse_rate(RateField::OldUnion, &input.old_union)?,
            new_member: parse_rate(RateField::NewMember, &input.new_member)?,
            new_society: parse_rate(RateField::NewSociety, &input.new_society)?,
            new_union: parse_rate(RateField::NewUnion, &input.new_union)?,
            new_from_month: input.new_from_month,
        },
    })
}

fn parse_rate(field: RateField, value: &str) -> Result<f64, SettingsValidationError> {
    let rate: f64 = value
        .trim()
        .parse()
        .map_err(|_| SettingsValidationError::InvalidRate {
            field,
            issue: RateValidationIssue::NotANumber,
        })?;
    if rate < 0.0 {
        return Err(SettingsValidationError::InvalidRate {
            field,
            issue: RateValidationIssue::Negative,
        });
    }
    Ok(rate)
}

fn is_excel_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "xls" | "xlsx"))
}

fn ensure_xlsx_extension(mut path: PathBuf) -> PathBuf {
    let has_xlsx = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"));
    if !has_xlsx {
        path.set_extension("xlsx");
    }
    path
}

fn valid_financial_year(value: &str) -> bool {
    let mut parts = value.trim().split('-');
    let (Some(start), Some(end), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    if start.len() != 4
        || end.len() != 2
        || !start.chars().all(|character| character.is_ascii_digit())
        || !end.chars().all(|character| character.is_ascii_digit())
    {
        return false;
    }
    let Ok(start): Result<u16, _> = start.parse() else {
        return false;
    };
    let Ok(end): Result<u16, _> = end.parse() else {
        return false;
    };
    (start + 1) % 100 == end
}

fn paths_match(left: &Path, right: &Path) -> bool {
    if let (Ok(left), Ok(right)) = (left.canonicalize(), right.canonicalize()) {
        return left == right;
    }
    left == right
}
