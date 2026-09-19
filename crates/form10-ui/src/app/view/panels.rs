fn source_step(app: &App, compact: bool) -> Element<'_, Message> {
    if matches!(app.operation, Operation::Parsing) {
        return busy_card(
            icons::FILE,
            "Reading file",
            "Checking columns and members...",
        );
    }

    if let Some(source) = &app.source {
        let filename = display_name(&source.path);
        let file_identity = row![
            container(icon(icons::FILE, 24, icon_accent))
                .width(44)
                .height(44)
                .center_x(Fill)
                .center_y(Fill)
                .style(soft_accent_style),
            column![
                heading_owned(filename, if compact { 17 } else { 19 }),
                text("Excel workbook · ready to convert")
                    .size(12)
                    .style(text_muted),
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
        let stats = source_stats(&source.data, compact);
        let actions = source_actions(app, compact);
        return container(column![file_header, stats, actions].spacing(16))
            .padding(if compact { 18 } else { 22 })
            .style(card_style)
            .into();
    }

    let choose_button = button(
        row![
            icon(icons::FOLDER, 18, icon_on_accent),
            text("Choose Excel file").font(font(Weight::Semibold))
        ]
        .spacing(9)
        .align_y(iced::Alignment::Center),
    )
    .on_press_maybe((!app.is_busy()).then_some(Message::BrowseSource))
    .padding([11, 18])
    .style(primary_button);
    let (title, subtitle) = if app.dragging_file {
        (
            "Release to import",
            "We’ll validate the workbook before continuing",
        )
    } else {
        ("Drop your workbook here", "or select it from your computer")
    };
    let drop_area = container(
        column![
            container(icon(icons::UPLOAD, 28, icon_accent))
                .width(56)
                .height(56)
                .center_x(Fill)
                .center_y(Fill)
                .style(soft_accent_style),
            heading(title, if compact { 22 } else { 26 }),
            text(subtitle).size(14).style(text_muted),
            choose_button,
            text("Excel .xls or .xlsx · processed only on this device")
                .size(12)
                .style(text_muted),
        ]
        .spacing(12)
        .align_x(iced::Alignment::Center),
    )
    .width(Fill)
    .padding(if compact { 28 } else { 38 })
    .style(move |theme| drop_zone_style(theme, app.dragging_file));

    column![
        section_title(
            "Import workbook",
            "Select the month-wise Excel file to begin."
        ),
        drop_area,
    ]
    .spacing(14)
    .into()
}

fn source_stats(source: &form10_core::SourceData, compact: bool) -> Element<'_, Message> {
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
    if compact {
        row![full_width_action(replace), full_width_action(remove)]
            .spacing(9)
            .into()
    } else {
        row![replace, remove]
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
    .width(Fill)
    .padding(if compact { 19 } else { 22 })
    .style(card_style);

    let rates: Element<'_, Message> = column![
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
    .spacing(10)
    .into();
    let rate_card = container(
        column![
            card_heading(icons::DETAILS, "Rates"),
            column![
                text("Apply new rates from").size(12).style(text_muted),
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
    .width(Fill)
    .padding(if compact { 19 } else { 22 })
    .style(card_style);

    let validation = match app.summary() {
        Ok(_) => validation_line(true, "Details ready"),
        Err(error) => validation_line(false, error),
    };
    let details_cards: Element<'_, Message> = if compact {
        column![society_card, rate_card].spacing(14).into()
    } else {
        row![society_card, rate_card].spacing(14).into()
    };
    column![
        section_title(
            "Review details",
            "Confirm the imported data and contribution rates."
        ),
        source_strip(source),
        details_cards,
        validation,
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
        return success_view(path, &source.data, compact);
    }

    let summary = app.summary();
    let valid = summary.is_ok();
    let (metrics, validation) = match summary {
        Ok(summary) => {
            let metrics: Element<'_, Message> = if compact {
                column![
                    metric_owned(icons::USERS, "Members", summary.member_count.to_string()),
                    metric_owned(
                        icons::DETAILS,
                        "Active subscriptions",
                        summary.active_subscriptions.to_string()
                    ),
                    metric_owned(
                        icons::CONVERT,
                        "Total contributions",
                        format!(
                            "₹{:.2}",
                            summary.member_contribution
                                + summary.society_contribution
                                + summary.union_contribution
                        )
                    ),
                ]
                .spacing(10)
                .into()
            } else {
                row![
                    metric_owned(icons::USERS, "Members", summary.member_count.to_string()),
                    metric_owned(
                        icons::DETAILS,
                        "Active subscriptions",
                        summary.active_subscriptions.to_string()
                    ),
                    metric_owned(
                        icons::CONVERT,
                        "Total contributions",
                        format!(
                            "₹{:.2}",
                            summary.member_contribution
                                + summary.society_contribution
                                + summary.union_contribution
                        )
                    ),
                ]
                .spacing(10)
                .into()
            };
            (metrics, validation_line(true, "Ready to convert"))
        }
        Err(error) => (
            source_stats(&source.data, compact),
            validation_line(false, error),
        ),
    };
    let convert = primary_action(
        if matches!(app.operation, Operation::ChoosingOutput) {
            "Choose location..."
        } else {
            "Convert and save"
        },
        icons::CONVERT,
        (valid && !app.is_busy()).then_some(Message::ChooseOutput),
    );
    let actions: Element<'_, Message> = if compact {
        full_width_action(convert)
    } else {
        row![Space::new().width(Fill), convert]
            .spacing(9)
            .align_y(iced::Alignment::Center)
            .into()
    };

    let output_header = row![
        container(icon(icons::SAVE, 21, icon_accent))
            .width(42)
            .height(42)
            .center_x(Fill)
            .center_y(Fill)
            .style(soft_accent_style),
        column![
            text("OUTPUT WORKBOOK")
                .size(10)
                .font(font(Weight::Bold))
                .style(text_muted),
            text("FORM-10.xlsx").size(16).font(font(Weight::Semibold)),
        ]
        .spacing(2),
    ]
    .spacing(11)
    .align_y(iced::Alignment::Center);

    column![
        section_title(
            "Export FORM-10",
            "Review the summary and choose where to save the workbook."
        ),
        container(column![output_header, metrics, validation, actions].spacing(18))
            .padding(if compact { 20 } else { 24 })
            .style(card_style),
    ]
    .spacing(17)
    .into()
}

fn success_view<'a>(
    path: &'a Path,
    source: &'a form10_core::SourceData,
    compact: bool,
) -> Element<'a, Message> {
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
