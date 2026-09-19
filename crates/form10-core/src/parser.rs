use std::path::Path;

use calamine::{Data, Reader, open_workbook_auto};

use crate::error::ParseError;
use crate::model::{MONTHS, Member, ReportingPeriod, SourceData};

pub(crate) fn parse_source(path: &Path) -> Result<SourceData, ParseError> {
    let mut workbook =
        open_workbook_auto(path).map_err(|error| ParseError::CouldNotOpenWorkbook {
            message: error.to_string(),
        })?;

    let sheet_names = workbook.sheet_names().to_owned();
    for sheet_name in sheet_names {
        let range = workbook.worksheet_range(&sheet_name).map_err(|error| {
            ParseError::CouldNotReadSheet {
                sheet_name: sheet_name.clone(),
                message: error.to_string(),
            }
        })?;

        if let Some(source) = parse_range(&sheet_name, &range) {
            return Ok(source);
        }
    }

    Err(ParseError::ProcurementTableNotFound)
}

fn parse_range(sheet_name: &str, range: &calamine::Range<Data>) -> Option<SourceData> {
    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_text).collect())
        .collect();

    let header_index = rows.iter().position(|row| {
        MONTHS.iter().any(|month| has_header(row, month))
            && (has_header(row, "M.NO")
                || has_header(row, "M NO")
                || has_header(row, "MEMBER NAME"))
    })?;

    let header = &rows[header_index];
    let member_no_column = find_column(header, &["M.NO", "M NO", "MEMBER NO", "MEMBER NUMBER"])?;
    let name_column = find_column(header, &["MEMBER NAME", "NAME"])?;
    let month_columns: [Option<usize>; 12] =
        std::array::from_fn(|month| find_column(header, &[MONTHS[month]]));

    let mut members = Vec::new();
    for row in rows.iter().skip(header_index + 1) {
        let code = row
            .get(member_no_column)
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        let name = row
            .get(name_column)
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        if !looks_like_member_code(code) || name.is_empty() || name.eq_ignore_ascii_case("total") {
            continue;
        }

        let active_months = std::array::from_fn(|month| {
            month_columns[month]
                .and_then(|column| row.get(column))
                .and_then(|value| parse_number(value))
                .is_some_and(|value| value > 0.0)
        });

        if active_months.iter().any(|active| *active) {
            members.push(Member {
                code: code.to_owned(),
                name: name.to_owned(),
                active_months,
            });
        }
    }

    let reporting_period = rows
        .iter()
        .flatten()
        .find_map(|cell| reporting_period_from_text(cell));
    let reporting_months = reporting_period
        .as_ref()
        .and_then(months_in_period)
        .unwrap_or_else(|| std::array::from_fn(|month| month_columns[month].is_some()));

    (!members.is_empty()).then(|| SourceData {
        sheet_name: sheet_name.to_owned(),
        members,
        financial_year: reporting_period
            .as_ref()
            .and_then(financial_year_from_period)
            .or_else(|| {
                rows.iter()
                    .flatten()
                    .find_map(|cell| financial_year_from_text(cell))
            }),
        reporting_period,
        reporting_months,
        dcmpu: labelled_value(&rows, &["DCMPU"]),
        district: labelled_value(&rows, &["DISTRICT"]),
        society: labelled_value(&rows, &["SOCIETY NAME", "SOCIETY"])
            .filter(|value| !normalize(value).starts_with("CODE"))
            .or_else(|| society_from_title(&rows, header_index)),
        society_code: labelled_value(&rows, &["SOCIETY CODE", "SOCIETY NO", "SOCIETY NUMBER"]),
    })
}

fn reporting_period_from_text(value: &str) -> Option<ReportingPeriod> {
    let dates = dates_from_text(value);
    (dates.len() >= 2).then(|| ReportingPeriod {
        start: dates[0].clone(),
        end: dates[1].clone(),
    })
}

fn dates_from_text(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_digit() && character != '/')
        .filter(|part| valid_date(part))
        .map(str::to_owned)
        .collect()
}

fn valid_date(value: &str) -> bool {
    let mut parts = value.split('/');
    let (Some(day), Some(month), Some(year), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    let (Ok(day), Ok(month), Ok(year)) = (
        day.parse::<u32>(),
        month.parse::<u32>(),
        year.parse::<i32>(),
    ) else {
        return false;
    };
    (1..=12).contains(&month) && (1..=days_in_month(month, year)).contains(&day)
}

fn months_in_period(period: &ReportingPeriod) -> Option<[bool; 12]> {
    let (_, start_month, start_year) = date_parts(&period.start)?;
    let (_, end_month, end_year) = date_parts(&period.end)?;
    let start = start_year * 12 + start_month as i32 - 1;
    let end = end_year * 12 + end_month as i32 - 1;
    let span = end - start;
    (0..12).contains(&span).then(|| {
        let mut months = [false; 12];
        for offset in 0..=span {
            let calendar_month = ((start + offset).rem_euclid(12) + 1) as u32;
            months[fiscal_month_index(calendar_month).expect("calendar month is valid") as usize] =
                true;
        }
        months
    })
}

fn financial_year_from_period(period: &ReportingPeriod) -> Option<String> {
    let (_, month, year) = date_parts(&period.start)?;
    let start_year = if month >= 4 { year } else { year - 1 };
    Some(format!("{start_year}-{:02}", (start_year + 1) % 100))
}

fn date_parts(value: &str) -> Option<(u32, u32, i32)> {
    let mut parts = value.split('/');
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

fn fiscal_month_index(month: u32) -> Option<i32> {
    (1..=12).contains(&month).then(|| ((month + 8) % 12) as i32)
}

fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.trim().to_owned(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.to_owned(),
        Data::Error(value) => value.to_string(),
    }
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn has_header(row: &[String], expected: &str) -> bool {
    let expected = normalize(expected);
    row.iter().any(|value| normalize(value) == expected)
}

fn find_column(row: &[String], names: &[&str]) -> Option<usize> {
    row.iter().position(|value| {
        let normalized = normalize(value);
        names.iter().any(|name| normalized == normalize(name))
    })
}

fn looks_like_member_code(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

fn parse_number(value: &str) -> Option<f64> {
    value.replace(',', "").trim().parse().ok()
}

fn financial_year_from_text(value: &str) -> Option<String> {
    let compact: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_digit() {
                character
            } else {
                ' '
            }
        })
        .collect();
    let years: Vec<i32> = compact
        .split_whitespace()
        .filter(|part| part.len() == 4)
        .filter_map(|part| part.parse().ok())
        .collect();
    years
        .windows(2)
        .find(|years| years[1] == years[0] + 1)
        .map(|years| format!("{}-{:02}", years[0], years[1] % 100))
}

fn labelled_value(rows: &[Vec<String>], labels: &[&str]) -> Option<String> {
    rows.iter().find_map(|row| {
        row.iter().enumerate().find_map(|(column, cell)| {
            labels.iter().find_map(|label| {
                let value = cell.trim();
                let matches_label = value
                    .get(..label.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(label));
                if !matches_label {
                    return None;
                }

                let inline_value = value_after_label(value, label);
                if !inline_value.is_empty() {
                    return Some(inline_value);
                }

                row.iter()
                    .skip(column + 1)
                    .map(|value| value.trim())
                    .find(|value| !value.is_empty())
                    .map(str::to_owned)
            })
        })
    })
}

fn value_after_label(value: &str, label: &str) -> String {
    value
        .get(label.len()..)
        .unwrap_or_default()
        .trim_start_matches(|character: char| {
            character == ':' || character == '-' || character.is_whitespace()
        })
        .trim()
        .to_owned()
}

fn society_from_title(rows: &[Vec<String>], header_index: usize) -> Option<String> {
    rows[..header_index]
        .iter()
        .rev()
        .flatten()
        .map(|value| value.trim())
        .find(|value| {
            !value.is_empty()
                && financial_year_from_text(value).is_none()
                && (normalize(value).contains("MPCS") || normalize(value).contains("SOCIETY"))
        })
        .map(str::to_owned)
}
