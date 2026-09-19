use std::path::Path;

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Formula, Workbook, Worksheet};

use crate::model::{ContributionKind, Rates, Settings, SourceData};

const DATA_START_ROW: u32 = 10;
const MONTH_START_COLUMN: u16 = 9;

pub(crate) fn generate_form10(
    source: &SourceData,
    settings: &Settings,
    path: &Path,
) -> Result<(), String> {
    let start_year = financial_year_start(&settings.financial_year)?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("FORM-10").map_err(display_error)?;
    write_sheet(worksheet, source, settings, start_year)?;
    workbook.save(path).map_err(display_error)
}

fn write_sheet(
    worksheet: &mut Worksheet,
    source: &SourceData,
    settings: &Settings,
    start_year: i32,
) -> Result<(), String> {
    let title = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center);
    let subtitle = Format::new()
        .set_font_size(11)
        .set_align(FormatAlign::Center);
    let centered = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let centered_bold = centered.clone().set_bold().set_text_wrap();
    let name_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter);
    let total_format = centered.clone().set_bold();
    let total_name_format = name_format.clone().set_bold();

    worksheet.set_landscape();
    worksheet.set_paper_size(5);
    worksheet.set_margins(0.25, 0.25, 0.75, 0.75, 0.3, 0.3);
    worksheet.set_column_width(0, 6.0).map_err(display_error)?;
    worksheet.set_column_width(1, 11.0).map_err(display_error)?;
    worksheet.set_column_width(2, 24.0).map_err(display_error)?;
    worksheet.set_column_width(3, 12.0).map_err(display_error)?;
    worksheet.set_column_width(4, 12.0).map_err(display_error)?;
    worksheet.set_column_width(5, 12.0).map_err(display_error)?;
    worksheet.set_column_width(6, 9.0).map_err(display_error)?;
    worksheet.set_column_width(7, 13.0).map_err(display_error)?;
    worksheet.set_column_width(8, 13.0).map_err(display_error)?;
    worksheet.set_column_width(9, 5.5).map_err(display_error)?;
    worksheet.set_column_width(10, 5.5).map_err(display_error)?;
    worksheet.set_column_width(11, 5.5).map_err(display_error)?;
    worksheet.set_column_width(12, 5.5).map_err(display_error)?;
    worksheet.set_column_width(13, 5.5).map_err(display_error)?;
    worksheet.set_column_width(14, 5.5).map_err(display_error)?;
    worksheet.set_column_width(15, 5.5).map_err(display_error)?;
    worksheet.set_column_width(16, 5.5).map_err(display_error)?;
    worksheet.set_column_width(17, 5.5).map_err(display_error)?;
    worksheet.set_column_width(18, 5.5).map_err(display_error)?;
    worksheet.set_column_width(19, 5.5).map_err(display_error)?;
    worksheet.set_column_width(20, 5.5).map_err(display_error)?;

    worksheet
        .merge_range(0, 0, 0, 20, "FORM-10", &title)
        .map_err(display_error)?;
    worksheet
        .merge_range(
            1,
            0,
            1,
            20,
            "(Register to be Maintained by the Society)",
            &subtitle,
        )
        .map_err(display_error)?;
    worksheet
        .merge_range(2, 0, 2, 20, "Personal Details of Subscriber", &subtitle)
        .map_err(display_error)?;
    worksheet
        .merge_range(3, 0, 3, 20, &period_label(source, settings), &subtitle)
        .map_err(display_error)?;

    write_metadata(
        worksheet,
        4,
        "NAME OF THE DCMPU",
        &settings.dcmpu,
        &centered,
        &name_format,
    )?;
    write_metadata(
        worksheet,
        5,
        "NAME OF THE DISTRICT",
        &settings.district,
        &centered,
        &name_format,
    )?;
    write_metadata(
        worksheet,
        6,
        "NAME OF THE SOCIETY",
        &settings.society,
        &centered,
        &name_format,
    )?;
    write_metadata(
        worksheet,
        7,
        "CODE NO",
        &settings.society_code,
        &centered,
        &name_format,
    )?;

    worksheet
        .write_with_format(8, 0, "S", &centered_bold)
        .map_err(display_error)?;
    worksheet
        .write_with_format(8, 1, "Subscriber", &centered_bold)
        .map_err(display_error)?;
    worksheet
        .write_with_format(8, 2, "Subscriber", &centered_bold)
        .map_err(display_error)?;
    worksheet
        .merge_range(8, 3, 8, 6, "Contribution", &centered_bold)
        .map_err(display_error)?;
    worksheet
        .write_with_format(8, 7, "Date of", &centered_bold)
        .map_err(display_error)?;
    worksheet
        .write_with_format(8, 8, "Date of", &centered_bold)
        .map_err(display_error)?;
    worksheet
        .merge_range(
            8,
            9,
            8,
            20,
            "Whether Subscription Paid for the month",
            &centered_bold,
        )
        .map_err(display_error)?;

    let headers = vec![
        "No".to_owned(),
        "Subscriber\nCode No".to_owned(),
        "Subscriber\nName".to_owned(),
        format_rate_header(
            "Member",
            settings.rates.old_member,
            settings.rates.new_member,
        ),
        format_rate_header(
            "Society",
            settings.rates.old_society,
            settings.rates.new_society,
        ),
        format_rate_header("Union", settings.rates.old_union, settings.rates.new_union),
        "Penalty".to_owned(),
        "Date of\nReceipt".to_owned(),
        "Date of\nRemoval".to_owned(),
        "APR".to_owned(),
        "MAY".to_owned(),
        "JUN".to_owned(),
        "JUL".to_owned(),
        "AUG".to_owned(),
        "SEP".to_owned(),
        "OCT".to_owned(),
        "NOV".to_owned(),
        "DEC".to_owned(),
        "JAN".to_owned(),
        "FEB".to_owned(),
        "MAR".to_owned(),
    ];
    for (column, header) in headers.iter().enumerate() {
        worksheet
            .write_with_format(9, column as u16, header, &centered_bold)
            .map_err(display_error)?;
    }

    for (index, member) in source.members.iter().enumerate() {
        let row = DATA_START_ROW + index as u32;
        worksheet
            .write_with_format(row, 0, (index + 1) as u32, &centered)
            .map_err(display_error)?;
        worksheet
            .write_with_format(row, 1, &member.code, &centered)
            .map_err(display_error)?;
        worksheet
            .write_with_format(row, 2, &member.name, &name_format)
            .map_err(display_error)?;

        write_contribution_formula(
            worksheet,
            row,
            3,
            source.active_months_for(member),
            settings,
            ContributionKind::Member,
            &centered,
        )?;
        write_contribution_formula(
            worksheet,
            row,
            4,
            source.active_months_for(member),
            settings,
            ContributionKind::Society,
            &centered,
        )?;
        write_contribution_formula(
            worksheet,
            row,
            5,
            source.active_months_for(member),
            settings,
            ContributionKind::Union,
            &centered,
        )?;
        worksheet
            .write_with_format(row, 6, 0, &centered)
            .map_err(display_error)?;
        worksheet
            .write_with_format(
                row,
                7,
                receipt_date(&source.active_months_for(member), start_year),
                &centered,
            )
            .map_err(display_error)?;
        worksheet
            .write_blank(row, 8, &centered)
            .map_err(display_error)?;

        for (month, active) in source.active_months_for(member).iter().enumerate() {
            if *active {
                worksheet
                    .write_with_format(row, MONTH_START_COLUMN + month as u16, "Y", &centered)
                    .map_err(display_error)?;
            } else {
                worksheet
                    .write_blank(row, MONTH_START_COLUMN + month as u16, &centered)
                    .map_err(display_error)?;
            }
        }
    }

    let total_row = DATA_START_ROW + source.members.len() as u32;
    worksheet
        .write_blank(total_row, 0, &total_format)
        .map_err(display_error)?;
    worksheet
        .write_blank(total_row, 1, &total_format)
        .map_err(display_error)?;
    worksheet
        .write_with_format(total_row, 2, "Total", &total_name_format)
        .map_err(display_error)?;
    for column in 3..=5 {
        let cached_result: f64 = source
            .members
            .iter()
            .map(|member| match column {
                3 => settings
                    .rates
                    .contribution(&source.active_months_for(member), ContributionKind::Member),
                4 => settings
                    .rates
                    .contribution(&source.active_months_for(member), ContributionKind::Society),
                5 => settings
                    .rates
                    .contribution(&source.active_months_for(member), ContributionKind::Union),
                _ => unreachable!(),
            })
            .sum();
        let formula = Formula::new(format!(
            "=SUM({}:{})",
            cell_reference(DATA_START_ROW, column),
            cell_reference(total_row - 1, column)
        ))
        .set_result(format_rate(cached_result));
        worksheet
            .write_formula_with_format(total_row, column, formula, &total_format)
            .map_err(display_error)?;
    }
    worksheet
        .write_with_format(total_row, 6, 0, &total_format)
        .map_err(display_error)?;
    for column in 7..=20 {
        worksheet
            .write_blank(total_row, column, &total_format)
            .map_err(display_error)?;
    }

    Ok(())
}

fn write_metadata(
    worksheet: &mut Worksheet,
    row: u32,
    label: &str,
    value: &str,
    label_format: &Format,
    value_format: &Format,
) -> Result<(), String> {
    worksheet
        .merge_range(row, 0, row, 2, label, label_format)
        .map_err(display_error)?;
    worksheet
        .merge_range(row, 3, row, 20, value, value_format)
        .map_err(display_error)?;
    Ok(())
}

fn format_rate_header(label: &str, old_rate: f64, new_rate: f64) -> String {
    if old_rate == new_rate {
        format!("{label}\n{}", format_rate(old_rate))
    } else {
        format!(
            "{label}\n{} / {}",
            format_rate(old_rate),
            format_rate(new_rate)
        )
    }
}

fn period_label(source: &SourceData, settings: &Settings) -> String {
    source
        .reporting_period
        .as_ref()
        .map(|period| format!("{} to {}", period.start, period.end))
        .unwrap_or_else(|| settings.financial_year.clone())
}

fn write_contribution_formula(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    active_months: [bool; 12],
    settings: &Settings,
    kind: ContributionKind,
    format: &Format,
) -> Result<(), String> {
    let formula = Formula::new(contribution_formula(row, &settings.rates, kind));
    let formula = formula.set_result(format_rate(
        settings.rates.contribution(&active_months, kind),
    ));
    worksheet
        .write_formula_with_format(row, column, formula, format)
        .map_err(display_error)?;
    Ok(())
}

fn contribution_formula(row: u32, rates: &Rates, kind: ContributionKind) -> String {
    let old_rate = rate_for(rates, kind, false);
    let new_rate = rate_for(rates, kind, true);
    let first = cell_reference(row, MONTH_START_COLUMN);
    let last = cell_reference(row, MONTH_START_COLUMN + 11);

    if rates.new_from_month == 0 {
        return format!("=COUNTIF({first}:{last},\"Y\")*{}", format_rate(old_rate));
    }
    if rates.new_from_month == 1 {
        return format!("=COUNTIF({first}:{last},\"Y\")*{}", format_rate(new_rate));
    }

    let old_end = cell_reference(row, MONTH_START_COLUMN + rates.new_from_month as u16 - 2);
    let new_start = cell_reference(row, MONTH_START_COLUMN + rates.new_from_month as u16 - 1);
    format!(
        "=(COUNTIF({first}:{old_end},\"Y\")*{})+(COUNTIF({new_start}:{last},\"Y\")*{})",
        format_rate(old_rate),
        format_rate(new_rate)
    )
}

fn rate_for(rates: &Rates, kind: ContributionKind, new_rate: bool) -> f64 {
    match (kind, new_rate) {
        (ContributionKind::Member, false) => rates.old_member,
        (ContributionKind::Society, false) => rates.old_society,
        (ContributionKind::Union, false) => rates.old_union,
        (ContributionKind::Member, true) => rates.new_member,
        (ContributionKind::Society, true) => rates.new_society,
        (ContributionKind::Union, true) => rates.new_union,
    }
}

fn cell_reference(row: u32, column: u16) -> String {
    format!("{}{}", column_letter(column), row + 1)
}

fn column_letter(column: u16) -> String {
    let mut number = column as usize + 1;
    let mut letters = String::new();
    while number > 0 {
        let remainder = (number - 1) % 26;
        letters.insert(0, (b'A' + remainder as u8) as char);
        number = (number - 1) / 26;
    }
    letters
}

fn format_rate(rate: f64) -> String {
    let text = format!("{rate:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn receipt_date(active_months: &[bool; 12], start_year: i32) -> String {
    let Some(last_month) = active_months.iter().rposition(|active| *active) else {
        return String::new();
    };
    let calendar_month = (last_month + 3) % 12 + 1;
    let year = if last_month <= 8 {
        start_year
    } else {
        start_year + 1
    };
    let days = days_in_month(year, calendar_month as u32);
    format!("{days:02}/{calendar_month:02}/{year}")
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => unreachable!("month is always valid"),
    }
}

fn financial_year_start(value: &str) -> Result<i32, String> {
    value
        .split('-')
        .next()
        .ok_or_else(|| {
            "Financial year must use the format YYYY-YY, for example 2025-26.".to_owned()
        })?
        .trim()
        .parse()
        .map_err(|_| "Financial year must use the format YYYY-YY, for example 2025-26.".to_owned())
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
