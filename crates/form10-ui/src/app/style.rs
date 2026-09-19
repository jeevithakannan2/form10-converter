use super::*;

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

pub(super) fn icon_default(theme: &Theme) -> Color {
    theme.palette().text
}

pub(super) fn icon_muted(theme: &Theme) -> Color {
    muted(theme)
}

pub(super) fn icon_accent(theme: &Theme) -> Color {
    accent(theme)
}

pub(super) fn icon_on_accent(_: &Theme) -> Color {
    Color::WHITE
}

pub(super) fn icon_success(theme: &Theme) -> Color {
    theme.palette().success
}

pub(super) fn icon_danger(theme: &Theme) -> Color {
    theme.palette().danger
}

pub(super) fn app_background(theme: &Theme) -> container::Style {
    container::Style::default().background(theme.palette().background)
}

pub(super) fn mark_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(accent_deep(theme))
        .border(Border::default().rounded(13))
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, if is_dark(theme) { 0.24 } else { 0.13 }),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        })
}

pub(super) fn card_style(theme: &Theme) -> container::Style {
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

pub(super) fn subtle_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(surface_soft(theme))
        .border(
            Border::default()
                .rounded(12)
                .width(1)
                .color(surface_border(theme)),
        )
}

pub(super) fn soft_accent_style(theme: &Theme) -> container::Style {
    let background = if is_dark(theme) {
        Color::from_rgb8(24, 59, 45)
    } else {
        Color::from_rgb8(228, 243, 235)
    };
    container::Style::default()
        .background(background)
        .border(Border::default().rounded(18))
}

pub(super) fn drop_zone_style(theme: &Theme, dragging: bool) -> container::Style {
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

pub(super) fn accent_panel_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(accent_deep(theme))
        .border(Border::default().rounded(20))
}

pub(super) fn accent_inset_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(Color::from_rgba8(255, 255, 255, 0.11))
        .border(
            Border::default()
                .rounded(15)
                .width(1)
                .color(Color::from_rgba8(255, 255, 255, 0.24)),
        )
}

pub(super) fn success_mark_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(if is_dark(theme) {
            Color::from_rgb8(31, 142, 91)
        } else {
            Color::from_rgb8(22, 128, 75)
        })
        .border(Border::default().rounded(99))
}

pub(super) fn success_panel_style(theme: &Theme) -> container::Style {
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

pub(super) fn notice_style(theme: &Theme, error: bool) -> container::Style {
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

pub(super) fn status_pill_style(theme: &Theme, success: bool) -> container::Style {
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

pub(super) fn working_pill_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(if is_dark(theme) {
            Color::from_rgb8(27, 65, 49)
        } else {
            Color::from_rgb8(227, 245, 235)
        })
        .color(accent(theme))
        .border(Border::default().rounded(99))
}

pub(super) fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
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

pub(super) fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
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

pub(super) fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
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

pub(super) fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: matches!(status, button::Status::Hovered | button::Status::Pressed)
            .then(|| Background::Color(surface_soft(theme))),
        text_color: theme.palette().text,
        border: Border::default().rounded(12),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub(super) fn tooltip_style(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(theme.palette().text)
        .color(theme.palette().background)
        .border(Border::default().rounded(7))
}

pub(super) fn text_muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(muted(theme)),
    }
}

pub(super) fn text_on_accent(_: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::WHITE),
    }
}

pub(super) fn text_on_accent_muted(_: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::from_rgb8(206, 235, 221)),
    }
}
