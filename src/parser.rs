use std::path::Path;

use calamine::{Data, Reader, open_workbook_auto};

use crate::model::{MONTHS, Member, SourceData};

pub fn parse_source(path: &Path) -> Result<SourceData, String> {
    let mut workbook =
        open_workbook_auto(path).map_err(|error| format!("Could not open workbook: {error}"))?;

    let sheet_names = workbook.sheet_names().to_owned();
    for sheet_name in sheet_names {
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|error| format!("Could not read sheet '{sheet_name}': {error}"))?;

        if let Some(source) = parse_range(&sheet_name, &range) {
            return Ok(source);
        }
    }

    Err("Could not find a month-wise procurement table. Expected headers for M.No, Member Name, and APR through MAR.".into())
}

fn parse_range(sheet_name: &str, range: &calamine::Range<Data>) -> Option<SourceData> {
    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_text).collect())
        .collect();

    let header_index = rows.iter().position(|row| {
        has_header(row, "APR")
            && has_header(row, "MAY")
            && has_header(row, "MAR")
            && (has_header(row, "M.NO")
                || has_header(row, "M NO")
                || has_header(row, "MEMBER NAME"))
    })?;

    let header = &rows[header_index];
    let member_no_column = find_column(header, &["M.NO", "M NO", "MEMBER NO", "MEMBER NUMBER"])?;
    let name_column = find_column(header, &["MEMBER NAME", "NAME"])?;
    let month_columns: [usize; 12] = std::array::from_fn(|month| {
        find_column(header, &[MONTHS[month]]).expect("validated all month headers")
    });

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
            row.get(month_columns[month])
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

    (!members.is_empty()).then(|| SourceData {
        sheet_name: sheet_name.to_owned(),
        members,
        financial_year: rows
            .iter()
            .flat_map(|row| row.iter())
            .find_map(|cell| financial_year_from_text(cell)),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_financial_year() {
        assert_eq!(
            financial_year_from_text("01/04/2025 to 31/03/2026"),
            Some("2025-26".into())
        );
    }

    #[test]
    fn recognizes_numeric_member_codes() {
        assert!(looks_like_member_code("0012"));
        assert!(!looks_like_member_code("Total"));
    }

    #[test]
    fn parses_supplied_month_wise_workbook() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("MONTH WISE MILK PROCUREMENTDETAIL -25-26.xls");
        let source = parse_source(&path).expect("the supplied input workbook should parse");
        assert_eq!(source.financial_year.as_deref(), Some("2025-26"));
        assert_eq!(source.members.len(), 40);
        assert_eq!(source.members[0].code, "3");
        assert!(source.members[0].active_months[0]);
    }
}
