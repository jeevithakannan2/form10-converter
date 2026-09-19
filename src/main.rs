mod generator;
mod icons;
mod model;
mod parser;
mod platform;

use std::path::{Path, PathBuf};

use iced::font::Weight;
use iced::widget::{
    Space, button, column, container, pick_list, responsive, row, scrollable, svg, text,
    text_input, tooltip,
};
use iced::{
    Background, Border, Color, Element, Fill, FillPortion, Font, Shadow, Size, Subscription, Task,
    Theme, Vector, window,
};
use rfd::AsyncFileDialog;

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

const WIDE_BREAKPOINT: f32 = 760.0;
const CONTENT_WIDTH: f32 = 1080.0;

fn main() -> iced::Result {
    iced::application(App::new, update, view)
        .title("Form 10 Converter")
        .theme(theme)
        .subscription(subscription)
        .default_font(Font::with_name("Fira Sans"))
        .antialiasing(true)
        .window(window::Settings {
            size: Size::new(1180.0, 780.0),
            min_size: Some(Size::new(680.0, 560.0)),
            icon: app_icon(),
            ..window::Settings::default()
        })
        .centered()
        .run()
}

fn app_icon() -> Option<window::Icon> {
    const SIZE: u32 = 64;
    let mut pixels = vec![0_u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let index = ((y * SIZE + x) * 4) as usize;
            let inside_sheet = (14..50).contains(&x) && (8..56).contains(&y);
            let fold = x >= 40 && y < 18 && y >= x - 32;
            let line = inside_sheet
                && ((22..26).contains(&y) || (33..37).contains(&y) || (44..48).contains(&y))
                && (21..43).contains(&x);
            let (red, green, blue, alpha) = if line || fold {
                (255, 255, 255, 255)
            } else if inside_sheet {
                (24, 128, 89, 255)
            } else {
                (18, 37, 29, 255)
            };
            pixels[index..index + 4].copy_from_slice(&[red, green, blue, alpha]);
        }
    }
    window::icon::from_rgba(pixels, SIZE, SIZE).ok()
}

fn theme(app: &App) -> Theme {
    match app.appearance {
        Appearance::Light => Theme::custom(
            "Form 10 Light",
            iced::theme::Palette {
                background: Color::from_rgb8(246, 245, 239),
                text: Color::from_rgb8(24, 35, 29),
                primary: Color::from_rgb8(21, 122, 84),
                success: Color::from_rgb8(21, 128, 61),
                warning: Color::from_rgb8(180, 83, 9),
                danger: Color::from_rgb8(190, 45, 45),
            },
        ),
        Appearance::Dark => Theme::custom(
            "Form 10 Dark",
            iced::theme::Palette {
                background: Color::from_rgb8(15, 21, 18),
                text: Color::from_rgb8(239, 243, 237),
                primary: Color::from_rgb8(72, 205, 148),
                success: Color::from_rgb8(74, 222, 128),
                warning: Color::from_rgb8(251, 191, 36),
                danger: Color::from_rgb8(248, 113, 113),
            },
        ),
    }
}

fn subscription(_: &App) -> Subscription<Message> {
    window::events().map(|(_, event)| match event {
        window::Event::FileHovered(path) => Message::FileHovered(path),
        window::Event::FilesHoveredLeft => Message::FilesHoveredLeft,
        window::Event::FileDropped(path) => Message::FileDropped(path),
        _ => Message::Noop,
    })
}

#[derive(Debug, Clone)]
enum Message {
    BrowseSource,
    SourceSelected(Option<PathBuf>),
    FileHovered(PathBuf),
    FilesHoveredLeft,
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
    ChooseOutput,
    OutputSelected(Option<PathBuf>),
    ExportFinished(Result<PathBuf, String>),
    OpenOutput,
    RevealOutput,
    ExternalActionFinished(Result<(), String>),
    StartAgain,
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
            Self::Source => "File",
            Self::Settings => "Details",
            Self::Export => "Convert",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Source => icons::FILE,
            Self::Settings => icons::DETAILS,
            Self::Export => icons::CONVERT,
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
            Self::AllOld => "All months: old rates",
            Self::April => "From April",
            Self::May => "From May",
            Self::June => "From June",
            Self::July => "From July",
            Self::August => "From August",
            Self::September => "From September",
            Self::October => "From October",
            Self::November => "From November",
            Self::December => "From December",
            Self::January => "From January",
            Self::February => "From February",
            Self::March => "From March",
        })
    }
}

#[derive(Debug)]
enum Operation {
    Idle,
    ChoosingSource,
    Parsing,
    ChoosingOutput,
    Exporting,
}

#[derive(Debug, Clone)]
enum Notice {
    Error(String),
}

struct App {
    active_step: Step,
    appearance: Appearance,
    operation: Operation,
    dragging_file: bool,
    source_path: Option<PathBuf>,
    source: Option<SourceData>,
    output_path: Option<PathBuf>,
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
    notice: Option<Notice>,
}

impl App {
    fn new() -> Self {
        Self {
            active_step: Step::Source,
            appearance: Appearance::Light,
            operation: Operation::Idle,
            dragging_file: false,
            source_path: None,
            source: None,
            output_path: None,
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
            notice: None,
        }
    }

    fn is_busy(&self) -> bool {
        !matches!(self.operation, Operation::Idle)
    }

    fn can_open(&self, step: Step) -> bool {
        match step {
            Step::Source => true,
            Step::Settings | Step::Export => self.source.is_some(),
        }
    }

    fn settings(&self) -> Result<Settings, String> {
        let parse_rate = |label: &str, value: &str| -> Result<f64, String> {
            let rate: f64 = value
                .trim()
                .parse()
                .map_err(|_| format!("Check {label}."))?;
            (rate >= 0.0)
                .then_some(rate)
                .ok_or_else(|| format!("{label} cannot be negative."))
        };

        if !valid_financial_year(&self.financial_year) {
            return Err("Use a year like 2025-26.".into());
        }
        if self.society.trim().is_empty() {
            return Err("Enter the society name.".into());
        }
        if self.society_code.trim().is_empty() {
            return Err("Enter the society code.".into());
        }

        Ok(Settings {
            financial_year: self.financial_year.trim().to_owned(),
            dcmpu: self.dcmpu.trim().to_owned(),
            district: self.district.trim().to_owned(),
            society: self.society.trim().to_owned(),
            society_code: self.society_code.trim().to_owned(),
            rates: Rates {
                old_member: parse_rate("old member rate", &self.old_member)?,
                old_society: parse_rate("old society rate", &self.old_society)?,
                old_union: parse_rate("old union rate", &self.old_union)?,
                new_member: parse_rate("new member rate", &self.new_member)?,
                new_society: parse_rate("new society rate", &self.new_society)?,
                new_union: parse_rate("new union rate", &self.new_union)?,
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
            app.operation = Operation::ChoosingSource;
            app.notice = None;
            Task::perform(
                async {
                    AsyncFileDialog::new()
                        .set_title("Choose Excel file")
                        .add_filter("Excel files", &["xls", "xlsx"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_owned())
                },
                Message::SourceSelected,
            )
        }
        Message::SourceSelected(path) => {
            app.operation = Operation::Idle;
            match path {
                Some(path) => begin_source_load(app, path),
                None => Task::none(),
            }
        }
        Message::FileHovered(path) => {
            if app.active_step == Step::Source && !app.is_busy() {
                app.dragging_file = is_excel_file(&path);
            }
            Task::none()
        }
        Message::FilesHoveredLeft => {
            app.dragging_file = false;
            Task::none()
        }
        Message::FileDropped(path) => {
            app.dragging_file = false;
            if app.active_step == Step::Source && !app.is_busy() {
                begin_source_load(app, path)
            } else {
                Task::none()
            }
        }
        Message::SourceLoaded(result) => {
            app.operation = Operation::Idle;
            match result {
                Ok((path, source)) => {
                    if let Some(financial_year) = &source.financial_year {
                        app.financial_year = financial_year.clone();
                    }
                    app.source_path = Some(path);
                    app.source = Some(source);
                    app.output_path = None;
                    app.notice = None;
                    app.active_step = Step::Settings;
                }
                Err(error) => app.notice = Some(Notice::Error(error)),
            }
            Task::none()
        }
        Message::RemoveSource => {
            if !app.is_busy() {
                app.source_path = None;
                app.source = None;
                app.output_path = None;
                app.notice = None;
                app.active_step = Step::Source;
            }
            Task::none()
        }
        Message::GoToStep(step) => {
            if app.can_open(step) && !app.is_busy() {
                if step == Step::Export
                    && let Err(error) = app.settings()
                {
                    app.notice = Some(Notice::Error(error));
                    app.active_step = Step::Settings;
                    return Task::none();
                }
                app.active_step = step;
                app.notice = None;
            }
            Task::none()
        }
        Message::ToggleAppearance => {
            app.appearance = match app.appearance {
                Appearance::Light => Appearance::Dark,
                Appearance::Dark => Appearance::Light,
            };
            Task::none()
        }
        Message::FinancialYearChanged(value) => change_field(app, |app| app.financial_year = value),
        Message::DcmpuChanged(value) => change_field(app, |app| app.dcmpu = value),
        Message::DistrictChanged(value) => change_field(app, |app| app.district = value),
        Message::SocietyChanged(value) => change_field(app, |app| app.society = value),
        Message::SocietyCodeChanged(value) => change_field(app, |app| app.society_code = value),
        Message::OldMemberChanged(value) => change_field(app, |app| app.old_member = value),
        Message::OldSocietyChanged(value) => change_field(app, |app| app.old_society = value),
        Message::OldUnionChanged(value) => change_field(app, |app| app.old_union = value),
        Message::NewMemberChanged(value) => change_field(app, |app| app.new_member = value),
        Message::NewSocietyChanged(value) => change_field(app, |app| app.new_society = value),
        Message::NewUnionChanged(value) => change_field(app, |app| app.new_union = value),
        Message::NewFromChanged(value) => {
            app.new_from = value;
            app.notice = None;
            app.output_path = None;
            Task::none()
        }
        Message::ChooseOutput => {
            if app.is_busy() || app.source.is_none() {
                return Task::none();
            }
            let settings = match app.settings() {
                Ok(settings) => settings,
                Err(error) => {
                    app.notice = Some(Notice::Error(error));
                    app.active_step = Step::Settings;
                    return Task::none();
                }
            };
            app.operation = Operation::ChoosingOutput;
            app.notice = None;
            Task::perform(
                async move {
                    AsyncFileDialog::new()
                        .set_title("Save FORM-10")
                        .set_file_name(format!("FORM-10-{}.xlsx", settings.financial_year))
                        .add_filter("Excel workbook", &["xlsx"])
                        .save_file()
                        .await
                        .map(|file| file.path().to_owned())
                },
                Message::OutputSelected,
            )
        }
        Message::OutputSelected(path) => {
            app.operation = Operation::Idle;
            let Some(path) = path else {
                return Task::none();
            };
            let Some(source) = app.source.clone() else {
                app.active_step = Step::Source;
                app.notice = Some(Notice::Error("Choose an Excel file first.".into()));
                return Task::none();
            };
            let settings = match app.settings() {
                Ok(settings) => settings,
                Err(error) => {
                    app.notice = Some(Notice::Error(error));
                    app.active_step = Step::Settings;
                    return Task::none();
                }
            };
            let path = ensure_xlsx_extension(path);
            if app.source_path.as_deref() == Some(path.as_path()) {
                app.notice = Some(Notice::Error(
                    "Choose a different name. The source file cannot be replaced.".into(),
                ));
                return Task::none();
            }
            app.operation = Operation::Exporting;
            app.notice = None;
            iced_runtime::task::blocking(move |mut sender| {
                let result = generate_form10(&source, &settings, &path)
                    .map(|()| path)
                    .map_err(|error| format!("Could not create the file: {error}"));
                let _ = sender.try_send(Message::ExportFinished(result));
            })
        }
        Message::ExportFinished(result) => {
            app.operation = Operation::Idle;
            match result {
                Ok(path) => {
                    app.output_path = Some(path);
                    app.notice = None;
                }
                Err(error) => app.notice = Some(Notice::Error(error)),
            }
            Task::none()
        }
        Message::OpenOutput => run_output_action(app, platform::open_file),
        Message::RevealOutput => run_output_action(app, platform::reveal_file),
        Message::ExternalActionFinished(result) => {
            if let Err(error) = result {
                app.notice = Some(Notice::Error(error));
            }
            Task::none()
        }
        Message::StartAgain => {
            if !app.is_busy() {
                app.source_path = None;
                app.source = None;
                app.output_path = None;
                app.notice = None;
                app.active_step = Step::Source;
            }
            Task::none()
        }
        Message::Noop => Task::none(),
    }
}

fn change_field(app: &mut App, change: impl FnOnce(&mut App)) -> Task<Message> {
    change(app);
    app.notice = None;
    app.output_path = None;
    Task::none()
}

fn begin_source_load(app: &mut App, path: PathBuf) -> Task<Message> {
    if !is_excel_file(&path) {
        app.notice = Some(Notice::Error("Choose an .xls or .xlsx file.".into()));
        return Task::none();
    }
    app.operation = Operation::Parsing;
    app.notice = None;
    iced_runtime::task::blocking(move |mut sender| {
        let result = parse_source(&path).map(|source| (path, source));
        let _ = sender.try_send(Message::SourceLoaded(result));
    })
}

fn run_output_action(app: &mut App, action: fn(&Path) -> Result<(), String>) -> Task<Message> {
    let Some(path) = app.output_path.clone() else {
        return Task::none();
    };
    Task::perform(
        async move { action(&path) },
        Message::ExternalActionFinished,
    )
}

fn view(app: &App) -> Element<'_, Message> {
    responsive(move |size| app_view(app, size.width < WIDE_BREAKPOINT)).into()
}

fn app_view(app: &App, compact: bool) -> Element<'_, Message> {
    let horizontal_padding = if compact { 18 } else { 34 };
    let mut page = column![header(app, compact), stepper(app, compact)]
        .spacing(if compact { 18 } else { 24 })
        .padding([22, horizontal_padding])
        .width(Fill)
        .max_width(CONTENT_WIDTH);

    if let Some(notice) = &app.notice {
        page = page.push(notice_banner(notice));
    }

    page = page.push(match app.active_step {
        Step::Source => source_step(app, compact),
        Step::Settings => settings_step(app, compact),
        Step::Export => export_step(app, compact),
    });

    container(scrollable(page).width(Fill))
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .style(app_background)
        .into()
}

fn header(app: &App, compact: bool) -> Element<'_, Message> {
    let mark = container(icon(icons::APP, 23, icon_on_accent))
        .width(44)
        .height(44)
        .center_x(Fill)
        .center_y(Fill)
        .style(mark_style);
    let mut name = column![heading("FORM 10", 18)].spacing(0);
    if !compact {
        name = name.push(text("Excel converter").size(12).style(text_muted));
    }
    let appearance_icon = match app.appearance {
        Appearance::Light => icons::MOON,
        Appearance::Dark => icons::SUN,
    };
    let appearance_label = match app.appearance {
        Appearance::Light => "Dark mode",
        Appearance::Dark => "Light mode",
    };
    let theme_button = tooltip(
        button(icon(appearance_icon, 19, icon_default))
            .on_press(Message::ToggleAppearance)
            .width(44)
            .height(44)
            .style(icon_button),
        appearance_label,
        tooltip::Position::Bottom,
    )
    .style(tooltip_style);

    row![mark, name, Space::new().width(Fill), theme_button]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .into()
}

fn stepper(app: &App, compact: bool) -> Element<'_, Message> {
    let mut steps = row![].spacing(if compact { 5 } else { 10 }).width(Fill);
    for step in [Step::Source, Step::Settings, Step::Export] {
        let current = step == app.active_step;
        let complete = app.active_step > step || (step == Step::Source && app.source.is_some());
        let enabled = app.can_open(step) && !app.is_busy();
        let badge_content: Element<'_, Message> = if complete {
            icon(icons::CHECK, 15, icon_on_accent)
        } else {
            text(step.number())
                .size(12)
                .font(font(Weight::Semibold))
                .into()
        };
        let badge = container(badge_content)
            .width(28)
            .height(28)
            .center_x(Fill)
            .center_y(Fill)
            .style(move |theme| step_badge_style(theme, current, complete));
        let content: Element<'_, Message> = if compact {
            column![badge, text(step.title()).size(11)]
                .spacing(5)
                .align_x(iced::Alignment::Center)
                .into()
        } else {
            row![
                badge,
                icon(step.icon(), 17, icon_default),
                text(step.title()).size(14)
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .into()
        };
        steps = steps.push(
            button(content)
                .on_press_maybe(enabled.then_some(Message::GoToStep(step)))
                .width(FillPortion(1))
                .height(if compact { 70 } else { 52 })
                .style(move |theme, status| step_button(theme, status, current)),
        );
    }
    container(steps)
        .padding(5)
        .style(stepper_shell_style)
        .into()
}

fn notice_banner(notice: &Notice) -> Element<'_, Message> {
    let (icon_data, message, error) = match notice {
        Notice::Error(message) => (icons::ALERT, message.as_str(), true),
    };
    container(
        row![
            icon(
                icon_data,
                19,
                if error { icon_danger } else { icon_success }
            ),
            text(message).size(14).width(Fill)
        ]
        .spacing(11)
        .align_y(iced::Alignment::Center),
    )
    .padding([13, 15])
    .style(move |theme| notice_style(theme, error))
    .into()
}

fn source_step(app: &App, compact: bool) -> Element<'_, Message> {
    if matches!(app.operation, Operation::Parsing) {
        return busy_card(
            icons::FILE,
            "Reading file",
            "Checking columns and members...",
        );
    }

    if let (Some(path), Some(source)) = (&app.source_path, &app.source) {
        let filename = display_name(path);
        let file_identity = row![
            container(icon(icons::FILE, 28, icon_accent))
                .width(52)
                .height(52)
                .center_x(Fill)
                .center_y(Fill)
                .style(soft_accent_style),
            column![
                heading_owned(filename, if compact { 18 } else { 21 }),
                text("Excel workbook").size(12).style(text_muted),
            ]
            .spacing(4)
            .width(Fill),
        ]
        .spacing(14)
        .align_y(iced::Alignment::Center);
        let file_header: Element<'_, Message> = if compact {
            column![file_identity, status_pill("READY", true)]
                .spacing(12)
                .into()
        } else {
            row![file_identity, status_pill("READY", true)]
                .align_y(iced::Alignment::Center)
                .into()
        };
        let stats = source_stats(source, compact);
        let actions = source_actions(app, compact);
        return container(column![file_header, stats, actions].spacing(22))
            .padding(if compact { 20 } else { 28 })
            .style(card_style)
            .into();
    }

    let choose_button = button(
        row![
            icon(icons::FOLDER, 19, icon_on_accent),
            text("Choose file").font(font(Weight::Semibold))
        ]
        .spacing(9)
        .align_y(iced::Alignment::Center),
    )
    .on_press_maybe((!app.is_busy()).then_some(Message::BrowseSource))
    .padding([13, 21])
    .style(primary_button);
    let (title, subtitle) = if app.dragging_file {
        ("Release to add file", "Ready to read this workbook")
    } else {
        ("Drop Excel file", "or choose a file")
    };
    let drop_area = container(
        column![
            container(icon(icons::UPLOAD, 38, icon_accent))
                .width(76)
                .height(76)
                .center_x(Fill)
                .center_y(Fill)
                .style(soft_accent_style),
            heading(title, if compact { 25 } else { 31 }),
            text(subtitle).size(14).style(text_muted),
            choose_button,
            text(".xls  /  .xlsx").size(12).style(text_muted),
        ]
        .spacing(14)
        .align_x(iced::Alignment::Center),
    )
    .width(Fill)
    .padding(if compact { 34 } else { 54 })
    .style(move |theme| drop_zone_style(theme, app.dragging_file));

    let hint = container(
        row![
            icon(icons::CONVERT, 19, icon_accent),
            text("Excel file").size(13),
            icon(icons::ARROW_RIGHT, 17, icon_muted),
            text("FORM-10 workbook")
                .size(13)
                .font(font(Weight::Semibold)),
        ]
        .spacing(9)
        .align_y(iced::Alignment::Center),
    )
    .padding([11, 14])
    .style(subtle_style);

    column![
        section_title("Select file", "Start with the month-wise Excel file."),
        drop_area,
        container(hint).width(Fill).center_x(Fill),
    ]
    .spacing(18)
    .into()
}

fn source_stats(source: &SourceData, compact: bool) -> Element<'_, Message> {
    let sheet = metric(icons::FILE, "Sheet", &source.sheet_name);
    let year = metric(
        icons::CALENDAR,
        "Year",
        source.financial_year.as_deref().unwrap_or("Not found"),
    );
    let members = metric_owned(icons::USERS, "Members", source.members.len().to_string());
    if compact {
        column![sheet, year, members].spacing(10).into()
    } else {
        row![sheet, year, members].spacing(10).into()
    }
}

fn source_actions(app: &App, compact: bool) -> Element<'_, Message> {
    let replace = secondary_action(
        "Replace",
        icons::REFRESH,
        (!app.is_busy()).then_some(Message::BrowseSource),
    );
    let remove = danger_action(
        "Remove",
        icons::TRASH,
        (!app.is_busy()).then_some(Message::RemoveSource),
    );
    let next = primary_action(
        "Next",
        icons::ARROW_RIGHT,
        (!app.is_busy()).then_some(Message::GoToStep(Step::Settings)),
    );
    if compact {
        column![
            full_width_action(next),
            row![full_width_action(replace), full_width_action(remove)].spacing(9)
        ]
        .spacing(9)
        .into()
    } else {
        row![replace, remove, Space::new().width(Fill), next]
            .spacing(9)
            .align_y(iced::Alignment::Center)
            .into()
    }
}

fn settings_step(app: &App, compact: bool) -> Element<'_, Message> {
    let Some(source) = app.source.as_ref() else {
        return missing_source();
    };
    let society_card = container(
        column![
            card_heading(icons::BUILDING, "Society"),
            form_pair(
                field(
                    "Year",
                    "2025-26",
                    &app.financial_year,
                    Message::FinancialYearChanged
                ),
                field("DCMPU", "ERODE", &app.dcmpu, Message::DcmpuChanged),
                compact,
            ),
            form_pair(
                field("District", "ERODE", &app.district, Message::DistrictChanged),
                field(
                    "Society",
                    "Society name",
                    &app.society,
                    Message::SocietyChanged
                ),
                compact,
            ),
            field(
                "Code",
                "15-10-00429",
                &app.society_code,
                Message::SocietyCodeChanged
            ),
        ]
        .spacing(15),
    )
    .padding(if compact { 19 } else { 24 })
    .style(card_style);

    let rates: Element<'_, Message> = if compact {
        column![
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
        .spacing(14)
        .into()
    } else {
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
        .spacing(14)
        .into()
    };
    let rate_card = container(
        column![
            card_heading(icons::DETAILS, "Rates"),
            column![
                text("New rates").size(12).style(text_muted),
                pick_list(
                    RATE_START_OPTIONS,
                    Some(app.new_from),
                    Message::NewFromChanged
                )
                .width(Fill),
            ]
            .spacing(6),
            rates,
        ]
        .spacing(15),
    )
    .padding(if compact { 19 } else { 24 })
    .style(card_style);

    let validation = match app.settings() {
        Ok(_) => validation_line(true, "Details ready"),
        Err(error) => validation_line(false, error),
    };
    let actions = back_next_actions(
        compact,
        Message::GoToStep(Step::Source),
        "Next",
        Message::GoToStep(Step::Export),
    );

    column![
        section_title("Check details", "Confirm before conversion."),
        source_strip(source, app.source_path.as_deref()),
        society_card,
        rate_card,
        validation,
        actions,
    ]
    .spacing(16)
    .into()
}

fn export_step(app: &App, compact: bool) -> Element<'_, Message> {
    let Some(source) = app.source.as_ref() else {
        return missing_source();
    };
    if matches!(app.operation, Operation::Exporting) {
        return busy_card(
            icons::SAVE,
            "Creating FORM-10",
            "Writing the Excel workbook...",
        );
    }
    if let Some(path) = app.output_path.as_deref() {
        return success_view(path, source, compact);
    }

    let conversion_content: Element<'_, Message> = if compact {
        column![
            conversion_node(icons::FILE, "SOURCE", "Excel file"),
            icon(icons::CONVERT, 25, icon_accent),
            conversion_node(icons::SAVE, "OUTPUT", "FORM-10.xlsx"),
        ]
        .spacing(15)
        .align_x(iced::Alignment::Center)
        .into()
    } else {
        row![
            conversion_node(icons::FILE, "SOURCE", "Excel file"),
            icon(icons::ARROW_RIGHT, 30, icon_accent),
            conversion_node(icons::SAVE, "OUTPUT", "FORM-10.xlsx"),
        ]
        .spacing(22)
        .align_y(iced::Alignment::Center)
        .into()
    };
    let conversion = container(conversion_content)
        .width(Fill)
        .padding(if compact { 24 } else { 34 })
        .style(accent_panel_style);

    let summary = source_stats(source, compact);
    let settings = app.settings();
    let valid = settings.is_ok();
    let validation = match settings {
        Ok(_) => validation_line(true, "Ready to convert"),
        Err(error) => validation_line(false, error),
    };
    let convert = primary_action(
        if matches!(app.operation, Operation::ChoosingOutput) {
            "Choose location..."
        } else {
            "Convert"
        },
        icons::CONVERT,
        (valid && !app.is_busy()).then_some(Message::ChooseOutput),
    );
    let back = secondary_action(
        "Back",
        icons::ARROW_LEFT,
        (!app.is_busy()).then_some(Message::GoToStep(Step::Settings)),
    );
    let actions: Element<'_, Message> = if compact {
        column![full_width_action(convert), full_width_action(back)]
            .spacing(9)
            .into()
    } else {
        row![back, Space::new().width(Fill), convert]
            .spacing(9)
            .align_y(iced::Alignment::Center)
            .into()
    };

    column![
        section_title("Create FORM-10", "Choose where to save the new file."),
        conversion,
        container(column![summary, validation, actions].spacing(20))
            .padding(if compact { 20 } else { 26 })
            .style(card_style),
    ]
    .spacing(17)
    .into()
}

fn success_view<'a>(path: &'a Path, source: &'a SourceData, compact: bool) -> Element<'a, Message> {
    let filename = display_name(path);
    let result = container(
        column![
            container(icon(icons::CHECK, 40, icon_on_accent))
                .width(76)
                .height(76)
                .center_x(Fill)
                .center_y(Fill)
                .style(success_mark_style),
            heading("File ready", if compact { 27 } else { 34 }),
            text(filename).size(16).font(font(Weight::Semibold)),
            text(path.to_string_lossy())
                .size(12)
                .style(text_muted)
                .width(Fill),
        ]
        .spacing(11)
        .align_x(iced::Alignment::Center),
    )
    .width(Fill)
    .padding(if compact { 28 } else { 40 })
    .style(success_panel_style);

    let open = primary_action("Open file", icons::OPEN, Some(Message::OpenOutput));
    let reveal = secondary_action("Show folder", icons::FOLDER, Some(Message::RevealOutput));
    let again = secondary_action("New file", icons::REFRESH, Some(Message::StartAgain));
    let actions: Element<'_, Message> = if compact {
        column![
            full_width_action(open),
            full_width_action(reveal),
            full_width_action(again)
        ]
        .spacing(9)
        .into()
    } else {
        row![open, reveal, again]
            .spacing(9)
            .align_y(iced::Alignment::Center)
            .into()
    };

    column![
        result,
        source_stats(source, compact),
        container(actions).width(Fill).center_x(Fill),
    ]
    .spacing(17)
    .into()
}

fn busy_card<'a>(
    icon_data: &'static str,
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, Message> {
    container(
        column![
            container(icon(icon_data, 36, icon_accent))
                .width(72)
                .height(72)
                .center_x(Fill)
                .center_y(Fill)
                .style(soft_accent_style),
            heading(title, 28),
            text(subtitle).size(14).style(text_muted),
            container(text("WORKING").size(11).font(font(Weight::Bold)))
                .padding([8, 13])
                .style(working_pill_style),
        ]
        .spacing(14)
        .align_x(iced::Alignment::Center),
    )
    .width(Fill)
    .padding(58)
    .style(card_style)
    .into()
}

fn source_strip<'a>(source: &'a SourceData, path: Option<&'a Path>) -> Element<'a, Message> {
    let filename = path
        .map(display_name)
        .unwrap_or_else(|| "Excel file".into());
    container(
        row![
            icon(icons::FILE, 21, icon_accent),
            column![
                text(filename).size(14).font(font(Weight::Semibold)),
                text(format!("{} members", source.members.len()))
                    .size(12)
                    .style(text_muted),
            ]
            .spacing(2)
            .width(Fill),
            icon(icons::CHECK, 18, icon_success),
        ]
        .spacing(11)
        .align_y(iced::Alignment::Center),
    )
    .padding([12, 14])
    .style(subtle_style)
    .into()
}

fn section_title<'a>(title: &'a str, subtitle: &'a str) -> Element<'a, Message> {
    column![
        heading(title, 28),
        text(subtitle).size(14).style(text_muted),
    ]
    .spacing(4)
    .into()
}

fn card_heading<'a>(icon_data: &'static str, title: &'a str) -> Element<'a, Message> {
    row![
        container(icon(icon_data, 19, icon_accent))
            .width(36)
            .height(36)
            .center_x(Fill)
            .center_y(Fill)
            .style(soft_accent_style),
        heading(title, 19),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}

fn form_pair<'a>(
    first: Element<'a, Message>,
    second: Element<'a, Message>,
    compact: bool,
) -> Element<'a, Message> {
    if compact {
        column![first, second].spacing(14).into()
    } else {
        row![first, second].spacing(14).into()
    }
}

fn rate_group<'a>(
    title: &'a str,
    member: &'a str,
    society: &'a str,
    union: &'a str,
) -> Element<'a, Message> {
    let old = title == "Old rates";
    container(
        column![
            text(title).size(14).font(font(Weight::Semibold)),
            row![
                mini_field(
                    "Member",
                    member,
                    if old {
                        Message::OldMemberChanged
                    } else {
                        Message::NewMemberChanged
                    }
                ),
                mini_field(
                    "Society",
                    society,
                    if old {
                        Message::OldSocietyChanged
                    } else {
                        Message::NewSocietyChanged
                    }
                ),
                mini_field(
                    "Union",
                    union,
                    if old {
                        Message::OldUnionChanged
                    } else {
                        Message::NewUnionChanged
                    }
                ),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .width(Fill)
    .padding(14)
    .style(subtle_style)
    .into()
}

fn metric<'a>(icon_data: &'static str, label: &'a str, value: &'a str) -> Element<'a, Message> {
    container(
        row![
            icon(icon_data, 19, icon_accent),
            column![
                text(label).size(11).style(text_muted),
                text(value).size(14).font(font(Weight::Semibold)),
            ]
            .spacing(2),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .padding([12, 14])
    .style(subtle_style)
    .into()
}

fn metric_owned(
    icon_data: &'static str,
    label: &'static str,
    value: String,
) -> Element<'static, Message> {
    container(
        row![
            icon(icon_data, 19, icon_accent),
            column![
                text(label).size(11).style(text_muted),
                text(value).size(14).font(font(Weight::Semibold)),
            ]
            .spacing(2),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .padding([12, 14])
    .style(subtle_style)
    .into()
}

fn field<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'static,
) -> Element<'a, Message> {
    column![
        text(label).size(12).style(text_muted),
        text_input(placeholder, value)
            .on_input(on_input)
            .padding(12)
            .size(14)
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
        text(label).size(11).style(text_muted),
        text_input("0", value)
            .on_input(on_input)
            .padding(10)
            .size(14)
            .width(Fill),
    ]
    .spacing(5)
    .width(Fill)
    .into()
}

fn validation_line(valid: bool, message: impl Into<String>) -> Element<'static, Message> {
    container(
        row![
            icon(
                if valid { icons::CHECK } else { icons::ALERT },
                18,
                if valid { icon_success } else { icon_danger }
            ),
            text(message.into()).size(13),
        ]
        .spacing(9)
        .align_y(iced::Alignment::Center),
    )
    .padding([11, 13])
    .style(move |theme| notice_style(theme, !valid))
    .into()
}

fn status_pill(label: &str, success: bool) -> Element<'_, Message> {
    container(text(label).size(10).font(font(Weight::Bold)))
        .padding([7, 10])
        .style(move |theme| status_pill_style(theme, success))
        .into()
}

fn conversion_node<'a>(
    icon_data: &'static str,
    label: &'a str,
    value: &'a str,
) -> Element<'a, Message> {
    container(
        column![
            icon(icon_data, 26, icon_on_accent),
            text(label)
                .size(10)
                .font(font(Weight::Bold))
                .style(text_on_accent_muted),
            text(value)
                .size(16)
                .font(font(Weight::Semibold))
                .style(text_on_accent),
        ]
        .spacing(7)
        .align_x(iced::Alignment::Center),
    )
    .width(190)
    .padding(18)
    .style(accent_inset_style)
    .into()
}

fn missing_source() -> Element<'static, Message> {
    container(
        column![
            icon(icons::FILE, 36, icon_accent),
            heading("Choose a file first", 24),
            primary_action(
                "Select file",
                icons::FOLDER,
                Some(Message::GoToStep(Step::Source))
            ),
        ]
        .spacing(16)
        .align_x(iced::Alignment::Center),
    )
    .width(Fill)
    .padding(48)
    .style(card_style)
    .into()
}

fn back_next_actions(
    compact: bool,
    back_message: Message,
    next_label: &'static str,
    next_message: Message,
) -> Element<'static, Message> {
    let back = secondary_action("Back", icons::ARROW_LEFT, Some(back_message));
    let next = primary_action(next_label, icons::ARROW_RIGHT, Some(next_message));
    if compact {
        column![full_width_action(next), full_width_action(back)]
            .spacing(9)
            .into()
    } else {
        row![back, Space::new().width(Fill), next].spacing(9).into()
    }
}

fn primary_action<'a>(
    label: &'a str,
    icon_data: &'static str,
    message: Option<Message>,
) -> Element<'a, Message> {
    button(
        row![
            text(label).font(font(Weight::Semibold)),
            icon(icon_data, 18, icon_on_accent),
        ]
        .spacing(9)
        .align_y(iced::Alignment::Center),
    )
    .on_press_maybe(message)
    .height(46)
    .padding([0, 18])
    .style(primary_button)
    .into()
}

fn full_width_action<'a>(action: Element<'a, Message>) -> Element<'a, Message> {
    container(action).width(Fill).center_x(Fill).into()
}

fn secondary_action<'a>(
    label: &'a str,
    icon_data: &'static str,
    message: Option<Message>,
) -> Element<'a, Message> {
    button(
        row![
            icon(icon_data, 17, icon_default),
            text(label).font(font(Weight::Medium)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .on_press_maybe(message)
    .height(46)
    .padding([0, 16])
    .style(secondary_button)
    .into()
}

fn danger_action<'a>(
    label: &'a str,
    icon_data: &'static str,
    message: Option<Message>,
) -> Element<'a, Message> {
    button(
        row![
            icon(icon_data, 17, icon_danger),
            text(label).font(font(Weight::Medium)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .on_press_maybe(message)
    .height(46)
    .padding([0, 16])
    .style(danger_button)
    .into()
}

fn heading(content: &str, size: u32) -> iced::widget::Text<'_, Theme, iced::Renderer> {
    text(content).size(size).font(font(Weight::Semibold))
}

fn heading_owned(content: String, size: u32) -> iced::widget::Text<'static, Theme, iced::Renderer> {
    text(content).size(size).font(font(Weight::Semibold))
}

fn font(weight: Weight) -> Font {
    Font {
        family: iced::font::Family::Name("Fira Sans"),
        weight,
        ..Font::DEFAULT
    }
}

fn icon(data: &'static str, size: u16, color: fn(&Theme) -> Color) -> Element<'static, Message> {
    svg(svg::Handle::from_memory(data.as_bytes()))
        .width(u32::from(size))
        .height(u32::from(size))
        .style(move |theme, _| svg::Style {
            color: Some(color(theme)),
        })
        .into()
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

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Excel file".into())
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

fn is_dark(theme: &Theme) -> bool {
    let color = theme.palette().background;
    color.r + color.g + color.b < 1.4
}

fn surface(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgb8(23, 31, 27)
    } else {
        Color::WHITE
    }
}

fn surface_soft(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgb8(28, 38, 33)
    } else {
        Color::from_rgb8(241, 244, 239)
    }
}

fn surface_border(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgb8(54, 68, 61)
    } else {
        Color::from_rgb8(217, 223, 216)
    }
}

fn muted(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgb8(167, 181, 173)
    } else {
        Color::from_rgb8(101, 116, 107)
    }
}

fn accent(theme: &Theme) -> Color {
    theme.palette().primary
}

fn accent_deep(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgb8(32, 139, 99)
    } else {
        Color::from_rgb8(17, 105, 72)
    }
}

fn icon_default(theme: &Theme) -> Color {
    theme.palette().text
}

fn icon_muted(theme: &Theme) -> Color {
    muted(theme)
}

fn icon_accent(theme: &Theme) -> Color {
    accent(theme)
}

fn icon_on_accent(_: &Theme) -> Color {
    Color::WHITE
}

fn icon_success(theme: &Theme) -> Color {
    theme.palette().success
}

fn icon_danger(theme: &Theme) -> Color {
    theme.palette().danger
}

fn app_background(theme: &Theme) -> container::Style {
    container::Style::default().background(theme.palette().background)
}

fn mark_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(accent_deep(theme))
        .border(Border::default().rounded(13))
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, if is_dark(theme) { 0.24 } else { 0.13 }),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        })
}

fn card_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(surface(theme))
        .border(
            Border::default()
                .rounded(18)
                .width(1)
                .color(surface_border(theme)),
        )
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, if is_dark(theme) { 0.20 } else { 0.055 }),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 26.0,
        })
}

fn subtle_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(surface_soft(theme))
        .border(
            Border::default()
                .rounded(12)
                .width(1)
                .color(surface_border(theme)),
        )
}

fn soft_accent_style(theme: &Theme) -> container::Style {
    let background = if is_dark(theme) {
        Color::from_rgb8(24, 59, 45)
    } else {
        Color::from_rgb8(228, 243, 235)
    };
    container::Style::default()
        .background(background)
        .border(Border::default().rounded(18))
}

fn drop_zone_style(theme: &Theme, dragging: bool) -> container::Style {
    let background = if dragging {
        if is_dark(theme) {
            Color::from_rgb8(21, 55, 41)
        } else {
            Color::from_rgb8(231, 247, 238)
        }
    } else {
        surface(theme)
    };
    container::Style::default()
        .background(background)
        .border(
            Border::default()
                .rounded(22)
                .width(if dragging { 3 } else { 2 })
                .color(if dragging {
                    accent(theme)
                } else {
                    surface_border(theme)
                }),
        )
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, if is_dark(theme) { 0.18 } else { 0.045 }),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        })
}

fn stepper_shell_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(surface_soft(theme))
        .border(Border::default().rounded(16))
}

fn step_badge_style(theme: &Theme, current: bool, complete: bool) -> container::Style {
    let active = current || complete;
    container::Style::default()
        .background(if active {
            accent_deep(theme)
        } else {
            surface_border(theme)
        })
        .color(if active { Color::WHITE } else { muted(theme) })
        .border(Border::default().rounded(99))
}

fn accent_panel_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(accent_deep(theme))
        .border(Border::default().rounded(20))
}

fn accent_inset_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(Color::from_rgba8(255, 255, 255, 0.11))
        .border(
            Border::default()
                .rounded(15)
                .width(1)
                .color(Color::from_rgba8(255, 255, 255, 0.24)),
        )
}

fn success_mark_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(if is_dark(theme) {
            Color::from_rgb8(31, 142, 91)
        } else {
            Color::from_rgb8(22, 128, 75)
        })
        .border(Border::default().rounded(99))
}

fn success_panel_style(theme: &Theme) -> container::Style {
    let background = if is_dark(theme) {
        Color::from_rgb8(20, 48, 35)
    } else {
        Color::from_rgb8(233, 247, 238)
    };
    container::Style::default().background(background).border(
        Border::default()
            .rounded(22)
            .width(1)
            .color(theme.palette().success),
    )
}

fn notice_style(theme: &Theme, error: bool) -> container::Style {
    let (background, border) = if error {
        if is_dark(theme) {
            (Color::from_rgb8(57, 27, 29), theme.palette().danger)
        } else {
            (Color::from_rgb8(254, 240, 240), theme.palette().danger)
        }
    } else if is_dark(theme) {
        (Color::from_rgb8(20, 48, 35), theme.palette().success)
    } else {
        (Color::from_rgb8(233, 247, 238), theme.palette().success)
    };
    container::Style::default()
        .background(background)
        .border(Border::default().rounded(12).width(1).color(border))
}

fn status_pill_style(theme: &Theme, success: bool) -> container::Style {
    let color = if success {
        theme.palette().success
    } else {
        theme.palette().warning
    };
    container::Style::default()
        .background(Color { a: 0.12, ..color })
        .color(color)
        .border(Border::default().rounded(99))
}

fn working_pill_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(if is_dark(theme) {
            Color::from_rgb8(27, 65, 49)
        } else {
            Color::from_rgb8(227, 245, 235)
        })
        .color(accent(theme))
        .border(Border::default().rounded(99))
}

fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => accent(theme),
        button::Status::Pressed => accent_deep(theme),
        button::Status::Disabled => Color {
            a: 0.42,
            ..accent_deep(theme)
        },
        button::Status::Active => accent_deep(theme),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: Border::default().rounded(12),
        shadow: if matches!(status, button::Status::Hovered) {
            Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.16),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            }
        } else {
            Shadow::default()
        },
        snap: false,
    }
}

fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let background = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        surface_soft(theme)
    } else {
        surface(theme)
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if matches!(status, button::Status::Disabled) {
            muted(theme)
        } else {
            theme.palette().text
        },
        border: Border::default()
            .rounded(12)
            .width(1)
            .color(surface_border(theme)),
        shadow: Shadow::default(),
        snap: false,
    }
}

fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let color = theme.palette().danger;
    let background = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        Color { a: 0.12, ..color }
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: color,
        border: Border::default().rounded(12),
        shadow: Shadow::default(),
        snap: false,
    }
}

fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: matches!(status, button::Status::Hovered | button::Status::Pressed)
            .then(|| Background::Color(surface_soft(theme))),
        text_color: theme.palette().text,
        border: Border::default().rounded(12),
        shadow: Shadow::default(),
        snap: false,
    }
}

fn step_button(theme: &Theme, status: button::Status, current: bool) -> button::Style {
    let background = if current {
        surface(theme)
    } else if matches!(status, button::Status::Hovered) {
        Color {
            a: 0.55,
            ..surface(theme)
        }
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if matches!(status, button::Status::Disabled) {
            muted(theme)
        } else {
            theme.palette().text
        },
        border: Border::default().rounded(12),
        shadow: if current {
            Shadow {
                color: Color::from_rgba8(0, 0, 0, if is_dark(theme) { 0.17 } else { 0.05 }),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            }
        } else {
            Shadow::default()
        },
        snap: false,
    }
}

fn tooltip_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(theme.palette().text)
        .color(theme.palette().background)
        .border(Border::default().rounded(7))
}

fn text_muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(muted(theme)),
    }
}

fn text_on_accent(_: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::WHITE),
    }
}

fn text_on_accent_muted(_: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::from_rgb8(206, 235, 221)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_semantic_financial_year() {
        assert!(valid_financial_year("2025-26"));
        assert!(valid_financial_year("2099-00"));
        assert!(!valid_financial_year("2025-27"));
        assert!(!valid_financial_year("25-26"));
    }

    #[test]
    fn keeps_valid_xlsx_extension() {
        assert_eq!(
            ensure_xlsx_extension(PathBuf::from("FORM-10.XLSX")),
            PathBuf::from("FORM-10.XLSX")
        );
    }

    #[test]
    fn normalizes_output_extension() {
        assert_eq!(
            ensure_xlsx_extension(PathBuf::from("FORM-10.xls")),
            PathBuf::from("FORM-10.xlsx")
        );
        assert_eq!(
            ensure_xlsx_extension(PathBuf::from("FORM-10")),
            PathBuf::from("FORM-10.xlsx")
        );
    }
}
