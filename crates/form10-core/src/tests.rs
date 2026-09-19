use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rust_xlsxwriter::Workbook;
use zip::ZipArchive;

use crate::error::{ExportError, PreviewError, SettingsValidationError};
use crate::parser::parse_source;
use crate::{ConverterService, SettingsInput};

#[derive(Debug)]
struct TestFile {
    path: PathBuf,
}

impl TestFile {
    fn new(label: &str, extension: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "form10-core-{label}-{}-{unique}.{extension}",
            std::process::id()
        ));
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn sample_settings_input() -> SettingsInput {
    SettingsInput {
        financial_year: "2025-26".into(),
        dcmpu: "ERODE".into(),
        district: "ERODE".into(),
        society: "ED 217 ODANILAI MPCS".into(),
        society_code: "15-10-00429".into(),
        old_member: "1".into(),
        old_society: "0.5".into(),
        old_union: "0.5".into(),
        new_member: "10".into(),
        new_society: "1".into(),
        new_union: "1".into(),
        new_from_month: 11,
    }
}

#[test]
fn summary_allows_missing_society_details() {
    let source_file = TestFile::new("source-without-society-details", "xlsx");
    write_source_fixture(source_file.path());
    let source = ConverterService::new()
        .import(source_file.path())
        .expect("source workbook should import");
    let mut settings = sample_settings_input();
    settings.dcmpu.clear();
    settings.district.clear();
    settings.society.clear();
    settings.society_code.clear();

    let summary = ConverterService::new()
        .summarize(&source.data, settings)
        .expect("society details should be optional");

    assert!(summary.settings.dcmpu.is_empty());
    assert!(summary.settings.district.is_empty());
    assert!(summary.settings.society.is_empty());
    assert!(summary.settings.society_code.is_empty());
}

fn write_source_fixture(path: &Path) {
    let mut workbook = Workbook::new();

    {
        let sheet = workbook.add_worksheet();
        sheet
            .set_name("Notes")
            .expect("notes sheet name should be valid");
        sheet
            .write_string(0, 0, "Ignore this sheet")
            .expect("notes cell should write");
    }

    {
        let sheet = workbook.add_worksheet();
        sheet
            .set_name("Procurement")
            .expect("source sheet name should be valid");
        sheet
            .write_string(0, 0, "MONTH WISE PROCUREMENT 01/04/2025 to 31/03/2026")
            .expect("financial year text should write");
        sheet
            .write_string(1, 0, "DCMPU: ERODE")
            .expect("dcmpu should write");
        sheet
            .write_string(1, 4, "District: ERODE")
            .expect("district should write");
        sheet
            .write_string(1, 8, "Society: ED 217 ODANILAI MPCS")
            .expect("society should write");
        sheet
            .write_string(1, 12, "Society Code: 15-10-00429")
            .expect("society code should write");

        let headers = [
            "M.No",
            "Member Name",
            "APR",
            "MAY",
            "JUN",
            "JUL",
            "AUG",
            "SEP",
            "OCT",
            "NOV",
            "DEC",
            "JAN",
            "FEB",
            "MAR",
        ];
        for (column, header) in headers.iter().enumerate() {
            sheet
                .write_string(2, column as u16, *header)
                .expect("header should write");
        }

        sheet.write_number(3, 0, 3).expect("code should write");
        sheet
            .write_string(3, 1, "Arun")
            .expect("member name should write");
        for column in 2..=13 {
            sheet
                .write_number(3, column, 1)
                .expect("active month should write");
        }

        sheet.write_number(4, 0, 4).expect("code should write");
        sheet
            .write_string(4, 1, "Banu")
            .expect("member name should write");
        sheet
            .write_number(4, 2, 7)
            .expect("active month should write");

        sheet.write_number(5, 0, 5).expect("code should write");
        sheet
            .write_string(5, 1, "Charu")
            .expect("member name should write");
        sheet
            .write_number(5, 2, 0)
            .expect("inactive month should write");

        sheet
            .write_string(6, 0, "Total")
            .expect("total label should write");
    }

    workbook.save(path).expect("fixture workbook should save");
}

fn write_partial_source_fixture(path: &Path, months: &[&str], period: &str) {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet
        .set_name("Procurement")
        .expect("source sheet name should be valid");
    sheet
        .write_string(0, 0, period)
        .expect("financial year text should write");

    for (column, header) in ["M.No", "Member Name"]
        .iter()
        .chain(months.iter())
        .enumerate()
    {
        sheet
            .write_string(2, column as u16, *header)
            .expect("header should write");
    }

    sheet.write_number(3, 0, 3).expect("code should write");
    sheet
        .write_string(3, 1, "Arun")
        .expect("member name should write");
    for column in 0..months.len() {
        sheet
            .write_number(3, (column + 2) as u16, 1)
            .expect("active month should write");
    }

    workbook.save(path).expect("fixture workbook should save");
}

fn read_zip_entry(path: &Path, entry_name: &str) -> String {
    let archive = fs::File::open(path).expect("output workbook should exist");
    let mut zip = ZipArchive::new(archive).expect("output should be a valid xlsx archive");
    let mut entry = String::new();
    zip.by_name(entry_name)
        .expect("zip entry should exist")
        .read_to_string(&mut entry)
        .expect("zip entry should read");
    entry
}

#[test]
fn parser_reads_generated_source_workbook() {
    let source_file = TestFile::new("source", "xlsx");
    write_source_fixture(source_file.path());

    let source = parse_source(source_file.path()).expect("generated source workbook should parse");

    assert_eq!(source.sheet_name, "Procurement");
    assert_eq!(source.financial_year.as_deref(), Some("2025-26"));
    assert_eq!(
        source
            .reporting_period
            .as_ref()
            .map(|period| (&period.start, &period.end)),
        Some((&"01/04/2025".to_owned(), &"31/03/2026".to_owned()))
    );
    assert!(source.reporting_months.iter().all(|month| *month));
    assert_eq!(source.dcmpu.as_deref(), Some("ERODE"));
    assert_eq!(source.district.as_deref(), Some("ERODE"));
    assert_eq!(source.society.as_deref(), Some("ED 217 ODANILAI MPCS"));
    assert_eq!(source.society_code.as_deref(), Some("15-10-00429"));
    assert_eq!(source.members.len(), 2);
    assert_eq!(source.members[0].code, "3");
    assert!(source.members[0].active_months.iter().all(|active| *active));
    assert_eq!(source.members[1].name, "Banu");
    assert!(source.members[1].active_months[0]);
    assert!(!source.members[1].active_months[1]);
}

#[test]
fn parser_detects_society_name_from_an_unlabelled_title() {
    let source_file = TestFile::new("society-title-source", "xlsx");

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet
        .write_string(0, 0, "ED1288 CHILLANKATTUPUDUR MPCS LTD.,")
        .expect("society title should write");
    sheet
        .write_string(
            1,
            0,
            "Month Wise Milk Procurement Details from 01/04/2025 to 31/03/2026",
        )
        .expect("financial year should write");
    for (column, header) in ["M.No", "Member Name", "APR"].iter().enumerate() {
        sheet
            .write_string(2, column as u16, *header)
            .expect("header should write");
    }
    sheet.write_number(3, 0, 3).expect("code should write");
    sheet.write_string(3, 1, "Arun").expect("name should write");
    sheet.write_number(3, 2, 1).expect("month should write");
    workbook
        .save(source_file.path())
        .expect("workbook should save");

    let source = parse_source(source_file.path()).expect("source workbook should parse");

    assert_eq!(source.financial_year.as_deref(), Some("2025-26"));
    assert_eq!(
        source.society.as_deref(),
        Some("ED1288 CHILLANKATTUPUDUR MPCS LTD.,")
    );
}

#[test]
fn parser_accepts_a_partial_year_with_last_three_month_headers() {
    let source_file = TestFile::new("partial-source", "xlsx");
    write_partial_source_fixture(
        source_file.path(),
        &["JAN", "FEB", "MAR"],
        "MONTH WISE PROCUREMENT 01/01/2026 to 31/03/2026",
    );

    let source = parse_source(source_file.path()).expect("partial source workbook should parse");

    assert_eq!(source.members.len(), 1);
    assert!(
        source.members[0].active_months[9..]
            .iter()
            .all(|active| *active)
    );
    assert!(
        source.members[0].active_months[..9]
            .iter()
            .all(|active| !*active)
    );
}

#[test]
fn summary_counts_only_the_months_present_in_a_partial_year() {
    let source_file = TestFile::new("partial-summary-source", "xlsx");
    write_partial_source_fixture(
        source_file.path(),
        &["OCT", "NOV", "DEC"],
        "MONTH WISE PROCUREMENT 01/10/2025 to 31/12/2025",
    );

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("partial source workbook should import");
    let summary = service
        .summarize(&imported.data, sample_settings_input())
        .expect("partial source summary should succeed");

    assert_eq!(summary.member_count, 1);
    assert_eq!(summary.active_subscriptions, 3);
    assert_eq!(summary.member_contribution, 3.0);
    assert_eq!(summary.society_contribution, 1.5);
    assert_eq!(summary.union_contribution, 1.5);
}

#[test]
fn parser_uses_the_date_range_to_limit_a_partial_reporting_period() {
    let source_file = TestFile::new("period-source", "xlsx");
    write_partial_source_fixture(
        source_file.path(),
        &["APR", "MAY", "JUN"],
        "MONTH WISE PROCUREMENT 01/04/2026 to 30/06/2026",
    );

    let source = parse_source(source_file.path()).expect("partial source workbook should parse");

    assert_eq!(source.financial_year.as_deref(), Some("2026-27"));
    assert_eq!(
        source
            .reporting_period
            .as_ref()
            .map(|period| (&period.start, &period.end)),
        Some((&"01/04/2026".to_owned(), &"30/06/2026".to_owned()))
    );
    assert_eq!(
        source.reporting_months,
        [
            true, true, true, false, false, false, false, false, false, false, false, false
        ]
    );
}

#[test]
fn summary_validates_without_an_output_path() {
    let source_file = TestFile::new("summary-source", "xlsx");
    write_source_fixture(source_file.path());

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("generated source workbook should import");
    let summary = service
        .summarize(&imported.data, sample_settings_input())
        .expect("summary should succeed");

    assert_eq!(summary.member_count, 2);
    assert_eq!(summary.active_subscriptions, 13);
    assert_eq!(summary.member_contribution, 31.0);
    assert_eq!(summary.society_contribution, 7.5);
    assert_eq!(summary.union_contribution, 7.5);
}

#[test]
fn preview_validates_and_normalizes_destination() {
    let source_file = TestFile::new("preview-source", "xlsx");
    write_source_fixture(source_file.path());

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("generated source workbook should import");
    let destination = source_file.path().with_file_name("FORM-10.xls");

    let preview = service
        .preview(&imported, sample_settings_input(), &destination)
        .expect("preview should succeed");

    assert_eq!(preview.destination_path, destination.with_extension("xlsx"));
    assert!(!preview.destination_exists);
    assert_eq!(preview.member_count, 2);
    assert_eq!(preview.active_subscriptions, 13);
    assert_eq!(preview.member_contribution, 31.0);
    assert_eq!(preview.society_contribution, 7.5);
    assert_eq!(preview.union_contribution, 7.5);
    assert_eq!(preview.suggested_file_name, "FORM-10-2025-26.xlsx");
    assert_eq!(preview.settings.financial_year, "2025-26");
}

#[test]
fn preview_rejects_invalid_financial_year() {
    let source_file = TestFile::new("preview-invalid-year", "xlsx");
    write_source_fixture(source_file.path());

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("generated source workbook should import");
    let mut settings = sample_settings_input();
    settings.financial_year = "2025-27".into();

    let error = service
        .preview(
            &imported,
            settings,
            source_file.path().with_file_name("FORM-10.xlsx"),
        )
        .expect_err("preview should reject invalid financial years");

    assert!(matches!(
        error,
        PreviewError::Validation(SettingsValidationError::InvalidFinancialYear)
    ));
    assert_eq!(error.to_string(), "Use a year like 2025-26.");
}

#[test]
fn preview_rejects_source_overwrite() {
    let source_file = TestFile::new("preview-overwrite", "xlsx");
    write_source_fixture(source_file.path());

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("generated source workbook should import");

    let error = service
        .preview(&imported, sample_settings_input(), source_file.path())
        .expect_err("preview should reject using the source file as output");

    assert!(matches!(error, PreviewError::SourceWouldBeOverwritten));
}

#[test]
fn export_blocks_existing_destination_without_overwrite() {
    let source_file = TestFile::new("export-source", "xlsx");
    write_source_fixture(source_file.path());
    let output_file = TestFile::new("existing-output", "xlsx");
    fs::write(output_file.path(), b"existing").expect("existing output marker should write");

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("generated source workbook should import");
    let preview = service
        .preview(&imported, sample_settings_input(), output_file.path())
        .expect("preview should succeed");

    assert!(preview.destination_exists);

    let error = service
        .export(&imported.data, &preview, false)
        .expect_err("export should refuse to overwrite existing output");

    assert!(matches!(error, ExportError::DestinationExists(path) if path == output_file.path()));
}

#[test]
fn export_generates_expected_workbook() {
    let source_file = TestFile::new("export-fixture", "xlsx");
    write_source_fixture(source_file.path());
    let output_file = TestFile::new("generated-output", "xlsx");

    let service = ConverterService::new();
    let imported = service
        .import(source_file.path())
        .expect("generated source workbook should import");
    let preview = service
        .preview(
            &imported,
            sample_settings_input(),
            output_file.path().with_extension("xls"),
        )
        .expect("preview should succeed");

    let output_path = service
        .export(&imported.data, &preview, true)
        .expect("export should generate a workbook");

    assert_eq!(output_path, output_file.path().with_extension("xlsx"));

    let sheet_xml = read_zip_entry(&output_path, "xl/worksheets/sheet1.xml");
    let shared_strings_xml = read_zip_entry(&output_path, "xl/sharedStrings.xml");
    assert!(sheet_xml.contains("(COUNTIF(J11:S11,\"Y\")*1)+(COUNTIF(T11:U11,\"Y\")*10)"));
    assert!(sheet_xml.contains("SUM(D11:D12)"));
    assert!(shared_strings_xml.contains("01/04/2025 to 31/03/2026"));
    assert!(shared_strings_xml.contains("ED 217 ODANILAI MPCS"));
}
