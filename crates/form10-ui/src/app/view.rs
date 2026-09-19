use iced::widget::column;

use super::style::*;
use super::*;
use crate::icons;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    responsive(move |size| app_view(app, size.width < WIDE_BREAKPOINT)).into()
}

fn app_view(app: &App, compact: bool) -> Element<'_, Message> {
    let horizontal_padding = if compact { 16 } else { 28 };
    let mut page = column![header(app, compact), workflow_steps(app, compact)]
        .spacing(if compact { 16 } else { 20 })
        .padding([18, horizontal_padding])
        .width(Fill)
        .max_width(CONTENT_WIDTH);

    if let Some(notice) = &app.notice {
        page = page.push(notice_banner(notice));
    }

    page = page.push(source_step(app, compact));
    if app.source.is_some() {
        page = page
            .push(settings_step(app, compact))
            .push(export_step(app, compact));
    }

    container(scrollable(page).width(Fill))
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .style(app_background)
        .into()
}

fn header(app: &App, compact: bool) -> Element<'_, Message> {
    let mark = container(icon(icons::APP, 20, icon_on_accent))
        .width(40)
        .height(40)
        .center_x(Fill)
        .center_y(Fill)
        .style(mark_style);
    let mut name = column![heading("Form 10 Converter", 17)].spacing(1);
    if !compact {
        name = name.push(
            text("Excel to statutory workbook")
                .size(12)
                .style(text_muted),
        );
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

    container(
        row![mark, name, Space::new().width(Fill), theme_button]
            .spacing(12)
            .align_y(iced::Alignment::Center),
    )
    .padding([10, 12])
    .style(header_style)
    .into()
}

fn workflow_steps(app: &App, compact: bool) -> Element<'_, Message> {
    let has_source = app.source.is_some();
    let exported = app.output_path.is_some();
    let import = workflow_step("1", "Import", !has_source, has_source);
    let details = workflow_step("2", "Review details", has_source && !exported, exported);
    let export = workflow_step("3", "Export", exported, exported);

    if compact {
        row![
            import,
            workflow_divider(),
            details,
            workflow_divider(),
            export
        ]
        .spacing(7)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        row![
            import,
            workflow_divider(),
            details,
            workflow_divider(),
            export
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .into()
    }
}

fn workflow_step<'a>(
    number: &'a str,
    label: &'a str,
    active: bool,
    complete: bool,
) -> Element<'a, Message> {
    let marker = if complete { "✓" } else { number };
    row![
        container(text(marker).size(12).font(font(Weight::Bold)))
            .width(28)
            .height(28)
            .center_x(Fill)
            .center_y(Fill)
            .style(move |theme| step_badge_style(theme, active, complete)),
        text(label)
            .size(12)
            .font(font(if active {
                Weight::Semibold
            } else {
                Weight::Medium
            }))
            .style(move |theme| step_text_style(theme, active || complete)),
    ]
    .spacing(7)
    .align_y(iced::Alignment::Center)
    .into()
}

fn workflow_divider() -> Element<'static, Message> {
    container(Space::new().width(Fill).height(1))
        .width(Fill)
        .height(1)
        .style(divider_style)
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

include!("view/panels.rs");
include!("view/components.rs");
