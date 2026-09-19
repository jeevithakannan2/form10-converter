use iced::widget::column;

use super::style::*;
use super::*;
use crate::icons;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    responsive(move |size| app_view(app, size.width < WIDE_BREAKPOINT)).into()
}

fn app_view(app: &App, compact: bool) -> Element<'_, Message> {
    let horizontal_padding = if compact { 18 } else { 34 };
    let mut page = column![header(app, compact)]
        .spacing(if compact { 18 } else { 24 })
        .padding([22, horizontal_padding])
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
