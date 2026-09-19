fn source_strip(source: &ImportedSource) -> Element<'_, Message> {
    let filename = display_name(&source.path);
    container(
        row![
            icon(icons::FILE, 21, icon_accent),
            column![
                text(filename).size(14).font(font(Weight::Semibold)),
                text(format!("{} members", source.data.members.len()))
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
        heading(title, 26),
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

fn missing_source() -> Element<'static, Message> {
    container(
        column![
            icon(icons::FILE, 36, icon_accent),
            heading("Choose a file first", 24),
            primary_action("Select file", icons::FOLDER, Some(Message::BrowseSource)),
        ]
        .spacing(16)
        .align_x(iced::Alignment::Center),
    )
    .width(Fill)
    .padding(48)
    .style(card_style)
    .into()
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
    .height(42)
    .padding([0, 16])
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
    .height(42)
    .padding([0, 14])
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
    .height(42)
    .padding([0, 14])
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

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Excel file".into())
}
