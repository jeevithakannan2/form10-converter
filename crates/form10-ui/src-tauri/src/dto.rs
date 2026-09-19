use std::path::Path;

use form10_core::{ConversionSummary, ImportedSource, SettingsInput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInputDto {
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

impl From<SettingsInputDto> for SettingsInput {
    fn from(value: SettingsInputDto) -> Self {
        Self {
            financial_year: value.financial_year,
            dcmpu: value.dcmpu,
            district: value.district,
            society: value.society,
            society_code: value.society_code,
            old_member: value.old_member,
            old_society: value.old_society,
            old_union: value.old_union,
            new_member: value.new_member,
            new_society: value.new_society,
            new_union: value.new_union,
            new_from_month: value.new_from_month,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfoDto {
    pub path: String,
    pub file_name: String,
    pub sheet_name: String,
    pub financial_year: Option<String>,
    pub reporting_period: Option<String>,
    pub reporting_months: String,
    pub dcmpu: Option<String>,
    pub district: Option<String>,
    pub society: Option<String>,
    pub society_code: Option<String>,
    pub member_count: usize,
}

impl From<&ImportedSource> for SourceInfoDto {
    fn from(source: &ImportedSource) -> Self {
        Self {
            path: source.path.display().to_string(),
            file_name: source
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Workbook")
                .to_owned(),
            sheet_name: source.data.sheet_name.clone(),
            financial_year: source.data.financial_year.clone(),
            reporting_period: source
                .data
                .reporting_period
                .as_ref()
                .map(|period| format!("{} to {}", period.start, period.end)),
            reporting_months: source
                .data
                .reporting_months
                .iter()
                .enumerate()
                .filter_map(|(month, included)| {
                    included.then_some(
                        [
                            "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC", "JAN",
                            "FEB", "MAR",
                        ][month],
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
            dcmpu: source.data.dcmpu.clone(),
            district: source.data.district.clone(),
            society: source.data.society.clone(),
            society_code: source.data.society_code.clone(),
            member_count: source.data.members.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDto {
    pub member_count: usize,
    pub active_subscriptions: usize,
    pub member_contribution: f64,
    pub society_contribution: f64,
    pub union_contribution: f64,
    pub total_contribution: f64,
}

impl From<ConversionSummary> for SummaryDto {
    fn from(summary: ConversionSummary) -> Self {
        let total_contribution =
            summary.member_contribution + summary.society_contribution + summary.union_contribution;
        Self {
            member_count: summary.member_count,
            active_subscriptions: summary.active_subscriptions,
            member_contribution: summary.member_contribution,
            society_contribution: summary.society_contribution,
            union_contribution: summary.union_contribution,
            total_contribution,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreviewDto {
    pub destination_path: String,
    pub destination_exists: bool,
    pub suggested_file_name: String,
}

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}
