use std::path::{Path, PathBuf};

use iced::font::Weight;
use iced::widget::{
    Space, button, container, pick_list, responsive, row, scrollable, svg, text, text_input,
    tooltip,
};
use iced::{
    Background, Border, Color, Element, Fill, Font, Shadow, Size, Subscription, Task, Theme,
    Vector, window,
};
use rfd::AsyncFileDialog;

use form10_core::{ConverterService, ImportedSource, SettingsInput};

use crate::platform;

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

const WIDE_BREAKPOINT: f32 = 820.0;
const CONTENT_WIDTH: f32 = 1040.0;

pub fn run() -> iced::Result {
    iced::application(App::new, update, view::view)
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
                background: Color::from_rgb8(247, 248, 247),
                text: Color::from_rgb8(24, 32, 28),
                primary: Color::from_rgb8(24, 119, 83),
                success: Color::from_rgb8(22, 128, 75),
                warning: Color::from_rgb8(180, 83, 9),
                danger: Color::from_rgb8(190, 45, 45),
            },
        ),
        Appearance::Dark => Theme::custom(
            "Form 10 Dark",
            iced::theme::Palette {
                background: Color::from_rgb8(16, 20, 18),
                text: Color::from_rgb8(239, 243, 241),
                primary: Color::from_rgb8(75, 210, 153),
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
    SourceLoaded(Result<ImportedSource, String>),
    RemoveSource,
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
    appearance: Appearance,
    operation: Operation,
    dragging_file: bool,
    source: Option<ImportedSource>,
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
            appearance: Appearance::Light,
            operation: Operation::Idle,
            dragging_file: false,
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

    fn settings_input(&self) -> SettingsInput {
        SettingsInput {
            financial_year: self.financial_year.clone(),
            dcmpu: self.dcmpu.clone(),
            district: self.district.clone(),
            society: self.society.clone(),
            society_code: self.society_code.clone(),
            old_member: self.old_member.clone(),
            old_society: self.old_society.clone(),
            old_union: self.old_union.clone(),
            new_member: self.new_member.clone(),
            new_society: self.new_society.clone(),
            new_union: self.new_union.clone(),
            new_from_month: self.new_from.month_index(),
        }
    }

    fn summary(&self) -> Result<form10_core::ConversionSummary, String> {
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| "Choose an Excel file first.".to_owned())?;
        ConverterService::new()
            .summarize(&source.data, self.settings_input())
            .map_err(|error| error.to_string())
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
            if !app.is_busy() {
                app.dragging_file = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        matches!(extension.to_ascii_lowercase().as_str(), "xls" | "xlsx")
                    });
            }
            Task::none()
        }
        Message::FilesHoveredLeft => {
            app.dragging_file = false;
            Task::none()
        }
        Message::FileDropped(path) => {
            app.dragging_file = false;
            if !app.is_busy() {
                begin_source_load(app, path)
            } else {
                Task::none()
            }
        }
        Message::SourceLoaded(result) => {
            app.operation = Operation::Idle;
            match result {
                Ok(source) => {
                    if let Some(financial_year) = &source.data.financial_year {
                        app.financial_year = financial_year.clone();
                    }
                    app.source = Some(source);
                    app.output_path = None;
                    app.notice = None;
                }
                Err(error) => app.notice = Some(Notice::Error(error)),
            }
            Task::none()
        }
        Message::RemoveSource => {
            if !app.is_busy() {
                app.source = None;
                app.output_path = None;
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
            let summary = match app.summary() {
                Ok(summary) => summary,
                Err(error) => {
                    app.notice = Some(Notice::Error(error));
                    return Task::none();
                }
            };
            app.operation = Operation::ChoosingOutput;
            app.notice = None;
            Task::perform(
                async move {
                    AsyncFileDialog::new()
                        .set_title("Save FORM-10")
                        .set_file_name(format!("FORM-10-{}.xlsx", summary.settings.financial_year))
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
                app.notice = Some(Notice::Error("Choose an Excel file first.".into()));
                return Task::none();
            };
            let preview =
                match ConverterService::new().preview(&source, app.settings_input(), &path) {
                    Ok(preview) => preview,
                    Err(error) => {
                        app.notice = Some(Notice::Error(error.to_string()));
                        return Task::none();
                    }
                };
            app.operation = Operation::Exporting;
            app.notice = None;
            iced_runtime::task::blocking(move |mut sender| {
                let result = ConverterService::new()
                    .export(&source.data, &preview, true)
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
                app.source = None;
                app.output_path = None;
                app.notice = None;
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
    app.operation = Operation::Parsing;
    app.notice = None;
    iced_runtime::task::blocking(move |mut sender| {
        let result = ConverterService::new()
            .import(&path)
            .map_err(|error| error.to_string());
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

mod style;
mod view;
