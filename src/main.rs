mod generator;
mod model;
mod parser;

use std::path::{Path, PathBuf};
use std::time::Instant;

use iced::widget::{
    button, column, container, pick_list, progress_bar, row, scrollable, text, text_input,
};
use iced::{
    Background, Border, Color, Element, Fill, Font, Shadow, Subscription, Task, Theme, Vector,
    window,
};
use rfd::FileDialog;

use generator::generate_form10;
use model::{Rates, Settings, SourceData};
use parser::parse_source;

const RATE_START_OPTIONS: [RateStart; 13] = [
    RateStart::AllOld,
    RateStart::April,
    RateStart::May,
    RateStart::June,
    RateStart::July,
    RateStart::August,
    RateStart::September,
    RateStart::October,
    RateStart::November,
    RateStart::December,
    RateStart::January,
    RateStart::February,
    RateStart::March,
];

fn main() -> iced::Result {
    iced::application(App::new, update, view)
        .theme(theme)
        .subscription(subscription)
        .default_font(Font::with_name("Avenir Next"))
        .antialiasing(true)
        .centered()
        .run()
}

fn theme(app: &App) -> Theme {
    match app.appearance {
        Appearance::Light => Theme::custom(
            "Form 10 Studio Light",
            iced::theme::Palette {
                background: Color::from_rgb8(247, 248, 252),
                text: Color::from_rgb8(22, 30, 46),
                primary: Color::from_rgb8(79, 70, 229),
                success: Color::from_rgb8(22, 163, 74),
                warning: Color::from_rgb8(217, 119, 6),
                danger: Color::from_rgb8(220, 38, 38),
            },
        ),
        Appearance::Dark => Theme::custom(
            "Form 10 Studio Dark",
            iced::theme::Palette {
                background: Color::from_rgb8(11, 16, 32),
                text: Color::from_rgb8(238, 242, 255),
                primary: Color::from_rgb8(129, 140, 248),
                success: Color::from_rgb8(74, 222, 128),
                warning: Color::from_rgb8(251, 191, 36),
                danger: Color::from_rgb8(248, 113, 113),
            },
        ),
    }
}

fn subscription(_: &App) -> Subscription<Message> {
    Subscription::batch([
        window::events().map(|(_, event)| match event {
            window::Event::FileDropped(path) => Message::FileDropped(path),
            _ => Message::Noop,
        }),
        window::frames().map(Message::Frame),
    ])
}

#[derive(Debug, Clone)]
enum Message {
    BrowseSource,
    SourceSelected(Option<PathBuf>),
    FileDropped(PathBuf),
    SourceLoaded(Result<(PathBuf, SourceData), String>),
    RemoveSource,
    GoToStep(Step),
    ToggleAppearance,
    FinancialYearChanged(String),
    DcmpuChanged(String),
    DistrictChanged(String),
    SocietyChanged(String),
    SocietyCodeChanged(String),
    OldMemberChanged(String),
    OldSocietyChanged(String),
    OldUnionChanged(String),
    NewMemberChanged(String),
    NewSocietyChanged(String),
    NewUnionChanged(String),
    NewFromChanged(RateStart),
    Export,
    ExportFinished(Result<PathBuf, String>),
    Frame(Instant),
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Source,
    Settings,
    Export,
}

impl Step {
    fn number(self) -> usize {
        match self {
            Self::Source => 1,
            Self::Settings => 2,
            Self::Export => 3,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Source => "Source workbook",
            Self::Settings => "Review settings",
            Self::Export => "Export FORM-10",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Appearance {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RateStart {
    AllOld,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
    January,
    February,
    March,
}

impl RateStart {
    fn month_index(self) -> u8 {
        match self {
            Self::AllOld => 0,
            Self::April => 1,
            Self::May => 2,
            Self::June => 3,
            Self::July => 4,
            Self::August => 5,
            Self::September => 6,
            Self::October => 7,
            Self::November => 8,
            Self::December => 9,
            Self::January => 10,
            Self::February => 11,
            Self::March => 12,
        }
    }
}

impl std::fmt::Display for RateStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::AllOld => "All months use old rates",
            Self::April => "New rates from April",
            Self::May => "New rates from May",
            Self::June => "New rates from June",
            Self::July => "New rates from July",
            Self::August => "New rates from August",
            Self::September => "New rates from September",
            Self::October => "New rates from October",
            Self::November => "New rates from November",
            Self::December => "New rates from December",
            Self::January => "New rates from January",
            Self::February => "New rates from February",
            Self::March => "New rates from March",
        })
    }
}

#[derive(Debug)]
enum Operation {
    Idle,
    Parsing,
    Exporting,
}

#[derive(Debug, Clone)]
enum Notice {
    Neutral(String),
    Success(String),
    Error(String),
}

struct App {
    active_step: Step,
    appearance: Appearance,
    operation: Operation,
    source_path: Option<PathBuf>,
    source: Option<SourceData>,
    financial_year: String,
    dcmpu: String,
    district: String,
    society: String,
    society_code: String,
    old_member: String,
    old_society: String,
    old_union: String,
    new_member: String,
    new_society: String,
    new_union: String,
    new_from: RateStart,
    notice: Notice,
    notice_started: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            active_step: Step::Source,
            appearance: Appearance::Light,
            operation: Operation::Idle,
            source_path: None,
            source: None,
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
            new_from: RateStart::April,
            notice: Notice::Neutral("Choose a month-wise procurement workbook to begin.".into()),
            notice_started: Instant::now(),
        }
    }

    fn is_busy(&self) -> bool {
        !matches!(self.operation, Operation::Idle)
    }

    fn set_notice(&mut self, notice: Notice) {
        self.notice = notice;
        self.notice_started = Instant::now();
    }

    fn can_open(&self, step: Step) -> bool {
        match step {
            Step::Source => true,
            Step::Settings | Step::Export => self.source.is_some(),
        }
    }

    fn should_show_notice(&self) -> bool {
        !matches!(&self.notice, Notice::Neutral(message) if message == "Choose a month-wise procurement workbook to begin.")
    }

    fn settings(&self) -> Result<Settings, String> {
        let parse_rate = |label: &str, value: &str| -> Result<f64, String> {
            let rate: f64 = value
                .trim()
                .parse()
                .map_err(|_| format!("{label} must be a valid number."))?;
            (rate >= 0.0)
                .then_some(rate)
                .ok_or_else(|| format!("{label} cannot be negative."))
        };

        if !valid_financial_year(&self.financial_year) {
            return Err("Financial year must use the format YYYY-YY, for example 2025-26.".into());
        }
        if self.society.trim().is_empty() || self.society_code.trim().is_empty() {
            return Err("Society name and society code are required.".into());
        }

        Ok(Settings {
            financial_year: self.financial_year.trim().to_owned(),
            dcmpu: self.dcmpu.trim().to_owned(),
            district: self.district.trim().to_owned(),
            society: self.society.trim().to_owned(),
            society_code: self.society_code.trim().to_owned(),
            rates: Rates {
                old_member: parse_rate("Old member rate", &self.old_member)?,
                old_society: parse_rate("Old society rate", &self.old_society)?,
                old_union: parse_rate("Old union rate", &self.old_union)?,
                new_member: parse_rate("New member rate", &self.new_member)?,
                new_society: parse_rate("New society rate", &self.new_society)?,
                new_union: parse_rate("New union rate", &self.new_union)?,
                new_from_month: self.new_from.month_index(),
            },
        })
    }
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::BrowseSource => {
            if app.is_busy() {
                return Task::none();
            }
            let path = FileDialog::new()
                .add_filter("Excel workbooks", &["xls", "xlsx"])
                .pick_file();
            return Task::done(Message::SourceSelected(path));
        }
        Message::SourceSelected(Some(path)) | Message::FileDropped(path) => {
            if app.is_busy() {
                return Task::none();
            }
            if !is_excel_file(&path) {
                app.set_notice(Notice::Error(
                    "Use a .xls or .xlsx procurement workbook.".into(),
                ));
                return Task::none();
            }
            app.operation = Operation::Parsing;
            app.set_notice(Notice::Neutral(
                "Reading workbook and identifying month-wise members...".into(),
            ));
            return Task::perform(
                async move { parse_source(&path).map(|source| (path, source)) },
                Message::SourceLoaded,
            );
        }
        Message::SourceSelected(None) => {}
        Message::SourceLoaded(result) => {
            app.operation = Operation::Idle;
            match result {
                Ok((path, source)) => {
                    if let Some(financial_year) = &source.financial_year {
                        app.financial_year = financial_year.clone();
                    }
                    app.set_notice(Notice::Success(format!(
                        "Workbook ready. Found {} active members in '{}'.",
                        source.members.len(),
                        source.sheet_name
                    )));
                    app.source_path = Some(path);
                    app.source = Some(source);
                    app.active_step = Step::Settings;
                }
                Err(error) => app.set_notice(Notice::Error(error)),
            }
        }
        Message::RemoveSource => {
            if !app.is_busy() {
                app.source_path = None;
                app.source = None;
                app.active_step = Step::Source;
                app.set_notice(Notice::Neutral(
                    "Choose a month-wise procurement workbook to begin.".into(),
                ));
            }
        }
        Message::GoToStep(step) => {
            if app.can_open(step) && !app.is_busy() {
                app.active_step = step;
            }
        }
        Message::ToggleAppearance => {
            app.appearance = match app.appearance {
                Appearance::Light => Appearance::Dark,
                Appearance::Dark => Appearance::Light,
            };
        }
        Message::FinancialYearChanged(value) => app.financial_year = value,
        Message::DcmpuChanged(value) => app.dcmpu = value,
        Message::DistrictChanged(value) => app.district = value,
        Message::SocietyChanged(value) => app.society = value,
        Message::SocietyCodeChanged(value) => app.society_code = value,
        Message::OldMemberChanged(value) => app.old_member = value,
        Message::OldSocietyChanged(value) => app.old_society = value,
        Message::OldUnionChanged(value) => app.old_union = value,
        Message::NewMemberChanged(value) => app.new_member = value,
        Message::NewSocietyChanged(value) => app.new_society = value,
        Message::NewUnionChanged(value) => app.new_union = value,
        Message::NewFromChanged(value) => app.new_from = value,
        Message::Export => {
            let Some(source) = app.source.clone() else {
                app.set_notice(Notice::Error(
                    "Choose a source workbook before exporting.".into(),
                ));
                app.active_step = Step::Source;
                return Task::none();
            };
            let settings = match app.settings() {
                Ok(settings) => settings,
                Err(error) => {
                    app.set_notice(Notice::Error(error));
                    app.active_step = Step::Settings;
                    return Task::none();
                }
            };
            let output = FileDialog::new()
                .set_file_name(format!("FORM-10-{}.xlsx", settings.financial_year))
                .add_filter("Excel workbook", &["xlsx"])
                .save_file();
            let Some(path) = output else {
                app.set_notice(Notice::Neutral(
                    "Export cancelled. Your settings are unchanged.".into(),
                ));
                return Task::none();
            };
            app.operation = Operation::Exporting;
            app.set_notice(Notice::Neutral(
                "Creating your FORM-10 workbook with formulas...".into(),
            ));
            return Task::perform(
                async move {
                    generate_form10(&source, &settings, &path)
                        .map(|()| path)
                        .map_err(|error| format!("Could not create workbook: {error}"))
                },
                Message::ExportFinished,
            );
        }
        Message::ExportFinished(result) => {
            app.operation = Operation::Idle;
            match result {
                Ok(path) => {
                    app.set_notice(Notice::Success(format!(
                        "FORM-10 workbook created at {}.",
                        path.display()
                    )));
                }
                Err(error) => app.set_notice(Notice::Error(error)),
            }
        }
        Message::Frame(at) => {
            let _ = at;
        }
        Message::Noop => {}
    }
    Task::none()
}

fn view(app: &App) -> Element<'_, Message> {
    let mut page = column![header(app), stepper(app)]
        .spacing(18)
        .padding([28, 40])
        .width(Fill)
        .max_width(1280);
    if app.should_show_notice() {
        page = page.push(notice_banner(app));
    }
    page = page.push(match app.active_step {
        Step::Source => source_step(app),
        Step::Settings => settings_step(app),
        Step::Export => export_step(app),
    });

    container(scrollable(page).width(Fill))
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .style(app_background)
        .into()
}

fn header(app: &App) -> Element<'_, Message> {
    let toggle_label = match app.appearance {
        Appearance::Light => "Dark mode",
        Appearance::Dark => "Light mode",
    };
    row![
        column![
            text("FORM10 / CONVERTER").size(12).style(text_brand),
            text("Turn annual procurement into FORM-10.").size(34),
            text("Upload once. Review the rules. Export a clean, formula-ready workbook.")
                .size(15)
                .style(text_muted),
        ]
        .spacing(6)
        .width(Fill),
        button(toggle_label)
            .on_press(Message::ToggleAppearance)
            .padding([9, 13])
            .style(quiet_button),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

fn stepper(app: &App) -> Element<'_, Message> {
    let steps = [Step::Source, Step::Settings, Step::Export];
    let mut content = row![].spacing(8).align_y(iced::Alignment::Center);
    for (index, step) in steps.iter().enumerate() {
        let enabled = app.can_open(*step) && !app.is_busy();
        let current = *step == app.active_step;
        let complete = app.active_step > *step || (*step == Step::Source && app.source.is_some());
        let label = if complete {
            "Done".to_owned()
        } else {
            step.number().to_string()
        };
        content = content.push(
            button(
                row![
                    container(text(label).size(13))
                        .padding(7)
                        .style(move |_| step_badge(current, complete)),
                    text(step.title()).size(14),
                ]
                .spacing(9)
                .align_y(iced::Alignment::Center),
            )
            .on_press_maybe(enabled.then_some(Message::GoToStep(*step)))
            .padding([9, 12])
            .style(move |theme, status| step_button(theme, status, current)),
        );
        if index < steps.len() - 1 {
            content = content.push(
                container(" ")
                    .width(28)
                    .height(1)
                    .style(move |_| connector_style(complete)),
            );
        }
    }
    content.into()
}

fn notice_banner(app: &App) -> Element<'_, Message> {
    let (title, message, style) = match &app.notice {
        Notice::Neutral(message) => (
            "Status",
            message.as_str(),
            notice_neutral as fn(&Theme) -> container::Style,
        ),
        Notice::Success(message) => (
            "Ready",
            message.as_str(),
            notice_success as fn(&Theme) -> container::Style,
        ),
        Notice::Error(message) => (
            "Attention",
            message.as_str(),
            notice_error as fn(&Theme) -> container::Style,
        ),
    };
    let progress = match app.operation {
        Operation::Parsing => Some("Reading source workbook"),
        Operation::Exporting => Some("Writing FORM-10 workbook"),
        Operation::Idle => None,
    };
    let age = app.notice_started.elapsed().as_secs_f32();
    let progress_value = 0.18 + ((age * 2.4).sin() + 1.0) * 0.32;
    let mut content = column![text(title).size(13), text(message).size(14)].spacing(3);
    if let Some(label) = progress {
        content = content
            .push(text(label).size(12).style(text_muted))
            .push(progress_bar(0.0..=1.0, progress_value));
    }
    container(content).padding(14).style(style).into()
}

fn source_step(app: &App) -> Element<'_, Message> {
    if let (Some(path), Some(source)) = (&app.source_path, &app.source) {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Workbook");
        return card(
            "Workbook selected",
            "The source is parsed and ready for a settings review.",
            column![
                text(filename).size(20),
                text(path.display().to_string()).style(text_muted),
                row![
                    metric("Sheet", &source.sheet_name),
                    metric(
                        "Financial year",
                        source.financial_year.as_deref().unwrap_or("Not detected")
                    ),
                    metric_owned("Active members", source.members.len().to_string()),
                ]
                .spacing(12),
                row![
                    button("Replace workbook")
                        .on_press_maybe((!app.is_busy()).then_some(Message::BrowseSource))
                        .style(secondary_button),
                    button("Remove")
                        .on_press_maybe((!app.is_busy()).then_some(Message::RemoveSource))
                        .style(quiet_button),
                    container(
                        button("Continue to settings")
                            .on_press_maybe(
                                (!app.is_busy()).then_some(Message::GoToStep(Step::Settings))
                            )
                            .style(primary_button)
                    )
                    .width(Fill)
                    .align_x(iced::alignment::Horizontal::Right),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(16),
        );
    }

    let picker = column![
        text("Drop your workbook here").size(23),
        text("or choose a file from your computer")
            .size(14)
            .style(text_muted),
        button("Browse files")
            .on_press_maybe((!app.is_busy()).then_some(Message::BrowseSource))
            .padding([12, 22])
            .style(primary_button),
        text("Supports .xls and .xlsx files")
            .size(12)
            .style(text_muted),
    ]
    .spacing(12)
    .align_x(iced::Alignment::Center);
    let intro = column![
        text("STEP 01 / SOURCE WORKBOOK").size(12).style(text_on_brand_muted),
        text("Bring in the annual procurement file.").size(32).style(text_on_brand),
        text("We identify April-to-March columns, include every active member, and build one FORM-10 sheet with live Excel formulas.").size(16).style(text_on_brand_muted),
    ]
    .spacing(12)
    .width(Fill);
    let checks = container(
        column![
            text("WE WILL DETECT").size(11).style(text_on_brand_muted),
            text("Member code & name\nMonthly procurement values\nFinancial year when available")
                .size(14)
                .style(text_on_brand),
        ]
        .spacing(8),
    )
    .padding([16, 18])
    .width(270)
    .style(brand_inset_style);
    let upload = container(picker)
        .width(Fill)
        .padding([48, 26])
        .center_x(Fill)
        .style(drop_zone_style);
    let requirements = row![
        source_chip("12 months", "April to March mapping"),
        source_chip("1 output", "A single FORM-10 sheet"),
        source_chip("Formula ready", "COUNTIF and total formulas"),
    ]
    .spacing(10)
    .wrap()
    .vertical_spacing(10);

    container(
        column![
            row![intro, checks]
                .spacing(28)
                .align_y(iced::Alignment::Center)
                .wrap()
                .vertical_spacing(20),
            upload,
            container(requirements)
                .padding(14)
                .width(Fill)
                .style(brand_panel_style),
        ]
        .spacing(28),
    )
    .padding(36)
    .width(Fill)
    .style(brand_panel_style)
    .into()
}

fn settings_step(app: &App) -> Element<'_, Message> {
    let Some(source) = app.source.as_ref() else {
        return empty_state(
            "Choose a workbook first",
            "The settings review will unlock after the source workbook is parsed.",
            Message::GoToStep(Step::Source),
        );
    };
    let details = card(
        "Society details",
        "These values appear in the generated FORM-10 header.",
        column![
            row![
                field(
                    "Financial year",
                    "2025-26",
                    &app.financial_year,
                    Message::FinancialYearChanged
                ),
                field("DCMPU", "ERODE", &app.dcmpu, Message::DcmpuChanged),
            ]
            .spacing(14),
            row![
                field("District", "ERODE", &app.district, Message::DistrictChanged),
                field(
                    "Society",
                    "Society name",
                    &app.society,
                    Message::SocietyChanged
                ),
            ]
            .spacing(14),
            field(
                "Society code",
                "15-10-00429",
                &app.society_code,
                Message::SocietyCodeChanged
            ),
        ]
        .spacing(14),
    );
    let rates = card(
        "Contribution schedule",
        "FORM-10 formulas count the marked months and apply the selected old/new rate boundary.",
        column![
            text("New rates start").size(13).style(text_muted),
            pick_list(
                RATE_START_OPTIONS,
                Some(app.new_from),
                Message::NewFromChanged
            )
            .width(Fill),
            row![
                rate_group(
                    "Old rates",
                    &app.old_member,
                    &app.old_society,
                    &app.old_union
                ),
                rate_group(
                    "New rates",
                    &app.new_member,
                    &app.new_society,
                    &app.new_union
                ),
            ]
            .spacing(16),
            row![
                button("Back")
                    .on_press(Message::GoToStep(Step::Source))
                    .style(quiet_button),
                container(
                    button("Review export")
                        .on_press_maybe((!app.is_busy()).then_some(Message::GoToStep(Step::Export)))
                        .style(primary_button)
                )
                .width(Fill)
                .align_x(iced::alignment::Horizontal::Right),
            ]
            .spacing(10),
        ]
        .spacing(12),
    );
    column![source_context(source), details, rates]
        .spacing(18)
        .into()
}

fn export_step(app: &App) -> Element<'_, Message> {
    let Some(source) = app.source.as_ref() else {
        return empty_state(
            "Choose a workbook first",
            "The export review will unlock after the source workbook is parsed.",
            Message::GoToStep(Step::Source),
        );
    };
    let settings = app.settings();
    let valid = settings.is_ok() && !app.is_busy();
    let schedule = app.new_from.to_string();
    card(
        "Ready to export",
        "The generated workbook will have one FORM-10 sheet with Y markers, COUNTIF contribution formulas, cached values, and totals.",
        column![
            row![
                metric_owned("Active members", source.members.len().to_string()),
                metric("Financial year", &app.financial_year),
                metric("Society", &app.society),
                metric_owned("Rate boundary", schedule),
            ]
            .spacing(12),
            validation_message(settings),
            row![
                button("Back to settings")
                    .on_press_maybe((!app.is_busy()).then_some(Message::GoToStep(Step::Settings)))
                    .style(quiet_button),
                container(
                    button("Export FORM-10 XLSX")
                        .on_press_maybe(valid.then_some(Message::Export))
                        .padding([13, 20])
                        .style(primary_button)
                )
                .width(Fill)
                .align_x(iced::alignment::Horizontal::Right),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(18),
    )
}

fn source_context(source: &SourceData) -> Element<'_, Message> {
    container(
        row![
            text("Source").size(13).style(text_muted),
            text(format!(
                "{} active members from {}",
                source.members.len(),
                source.sheet_name
            ))
            .size(14),
        ]
        .spacing(8),
    )
    .padding([10, 12])
    .style(subtle_style)
    .into()
}

fn rate_group<'a>(
    title: &'a str,
    member: &'a str,
    society: &'a str,
    union: &'a str,
) -> Element<'a, Message> {
    column![
        text(title).size(16),
        row![
            mini_field(
                "Member",
                member,
                if title == "Old rates" {
                    Message::OldMemberChanged
                } else {
                    Message::NewMemberChanged
                }
            ),
            mini_field(
                "Society",
                society,
                if title == "Old rates" {
                    Message::OldSocietyChanged
                } else {
                    Message::NewSocietyChanged
                }
            ),
            mini_field(
                "Union",
                union,
                if title == "Old rates" {
                    Message::OldUnionChanged
                } else {
                    Message::NewUnionChanged
                }
            ),
        ]
        .spacing(9),
    ]
    .spacing(8)
    .width(Fill)
    .into()
}

fn metric<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
    container(column![text(label).size(12).style(text_muted), text(value).size(16)].spacing(3))
        .width(Fill)
        .padding(12)
        .style(subtle_style)
        .into()
}

fn metric_owned(label: &'static str, value: String) -> Element<'static, Message> {
    container(column![text(label).size(12).style(text_muted), text(value).size(16)].spacing(3))
        .width(Fill)
        .padding(12)
        .style(subtle_style)
        .into()
}

fn source_chip<'a>(title: &'a str, subtitle: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(title).size(14).style(text_on_brand),
            text(subtitle).size(12).style(text_on_brand_muted),
        ]
        .spacing(3),
    )
    .padding([11, 13])
    .width(220)
    .style(brand_inset_style)
    .into()
}

fn card<'a>(
    title: &'a str,
    subtitle: &'a str,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            text(title).size(22),
            text(subtitle).size(14).style(text_muted),
            body.into()
        ]
        .spacing(12),
    )
    .padding(22)
    .style(card_style)
    .into()
}

fn empty_state<'a>(title: &'a str, subtitle: &'a str, action: Message) -> Element<'a, Message> {
    card(
        title,
        subtitle,
        button("Go to source workbook")
            .on_press(action)
            .style(primary_button),
    )
}

fn validation_message(settings: Result<Settings, String>) -> Element<'static, Message> {
    match settings {
        Ok(_) => container(text("All required settings are valid.").style(text_success)).into(),
        Err(error) => container(text(error).style(text_danger)).into(),
    }
}

fn field<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'static,
) -> Element<'a, Message> {
    column![
        text(label).size(13).style(text_muted),
        text_input(placeholder, value)
            .on_input(on_input)
            .padding(11)
            .width(Fill),
    ]
    .spacing(6)
    .width(Fill)
    .into()
}

fn mini_field<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'static,
) -> Element<'a, Message> {
    column![
        text(label).size(12).style(text_muted),
        text_input("0", value)
            .on_input(on_input)
            .padding(10)
            .width(Fill),
    ]
    .spacing(5)
    .width(Fill)
    .into()
}

fn is_excel_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "xls" | "xlsx"))
}

fn valid_financial_year(value: &str) -> bool {
    let mut parts = value.trim().split('-');
    let Some(start) = parts.next() else {
        return false;
    };
    let Some(end) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && start.len() == 4
        && end.len() == 2
        && start.chars().all(|character| character.is_ascii_digit())
        && end.chars().all(|character| character.is_ascii_digit())
}

fn app_background(theme: &Theme) -> container::Style {
    container::Style::default().background(theme.palette().background)
}

fn card_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.background.base.color)
        .border(
            Border::default()
                .rounded(14)
                .width(1)
                .color(palette.background.strong.color),
        )
        .shadow(Shadow {
            color: Color {
                a: 0.07,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 8.0),
            blur_radius: 26.0,
        })
}

fn subtle_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.background.weak.color)
        .border(
            Border::default()
                .rounded(10)
                .width(1)
                .color(palette.background.strong.color),
        )
}

fn drop_zone_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.background.base.color)
        .border(
            Border::default()
                .rounded(0)
                .width(1)
                .color(palette.background.strong.color),
        )
}

fn brand_panel_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.primary.base.color)
        .border(Border::default().rounded(18))
}

fn brand_inset_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(Color::from_rgba8(255, 255, 255, 0.15))
        .border(
            Border::default()
                .rounded(10)
                .width(1)
                .color(Color::from_rgba8(255, 255, 255, 0.23)),
        )
}

fn notice_neutral(theme: &Theme) -> container::Style {
    subtle_style(theme)
}
fn notice_success(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.success.weak.color)
        .border(
            Border::default()
                .rounded(12)
                .width(1)
                .color(palette.success.strong.color),
        )
}
fn notice_error(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.danger.weak.color)
        .border(
            Border::default()
                .rounded(12)
                .width(1)
                .color(palette.danger.strong.color),
        )
}
fn connector_style(complete: bool) -> container::Style {
    let color = if complete {
        Color::from_rgb8(79, 70, 229)
    } else {
        Color::from_rgb8(203, 213, 225)
    };
    container::Style::default().background(Background::Color(color))
}
fn step_badge(current: bool, complete: bool) -> container::Style {
    let color = if current || complete {
        Color::from_rgb8(79, 70, 229)
    } else {
        Color::from_rgb8(203, 213, 225)
    };
    container::Style::default()
        .background(color)
        .color(Color::WHITE)
        .border(Border::default().rounded(99))
}
fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let color = match status {
        button::Status::Hovered => palette.primary.strong.color,
        button::Status::Pressed => palette.primary.base.color,
        _ => palette.primary.base.color,
    };
    button::Style {
        background: Some(Background::Color(color)),
        text_color: palette.primary.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
        snap: false,
    }
}
fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let background = match status {
        button::Status::Hovered => palette.primary.weak.color,
        _ => palette.background.weak.color,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.primary.strong.color,
        border: Border::default()
            .rounded(10)
            .width(1)
            .color(palette.primary.strong.color),
        shadow: Shadow::default(),
        snap: false,
    }
}
fn quiet_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let background = matches!(status, button::Status::Hovered)
        .then_some(Background::Color(palette.background.weak.color));
    button::Style {
        background,
        text_color: palette.background.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
        snap: false,
    }
}
fn step_button(theme: &Theme, status: button::Status, current: bool) -> button::Style {
    let palette = theme.extended_palette();
    let background = if current || matches!(status, button::Status::Hovered) {
        Some(Background::Color(palette.primary.weak.color))
    } else {
        None
    };
    button::Style {
        background,
        text_color: palette.background.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
        snap: false,
    }
}
fn text_muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.strong.text),
    }
}
fn text_brand(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().primary),
    }
}
fn text_on_brand(_: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::WHITE),
    }
}
fn text_on_brand_muted(_: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::from_rgb8(224, 231, 255)),
    }
}
fn text_success(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().success.strong.color),
    }
}
fn text_danger(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().danger.strong.color),
    }
}
