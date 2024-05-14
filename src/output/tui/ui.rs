use std::os::macos::raw::stat;

use contract_transcode::Value;
use ratatui::{
    layout::{
        Constraint,
        Direction,
        Layout,
        Rect,
    },
    prelude::*,
    style::{
        Color,
        Style,
    },
    symbols::border,
    text::{
        Line,
        Span,
        Text,
    },
    widgets::{
        block::{
            Position,
            Title,
        },
        Block,
        BorderType,
        Borders,
        Cell,
        Clear,
        HighlightSpacing,
        List,
        ListItem,
        Paragraph,
        Row,
        Scrollbar,
        ScrollbarOrientation,
        Table,
        Wrap,
    },
    Frame,
};

use crate::engine::{
    CampaignStatus,
    DeployOrMessage,
    FailReason,
};

use super::app::{
    App,
    ITEM_HEIGHT,
};

const INFO_TEXT_OPEN_MODAL: &str =
    "(Q) Quit | (↑) Move up | (↓) Move down | (ENTER) Open Failed Trace";

const INFO_TEXT_CLOSE_MODAL: &str =
    "(Q) Quit | (↑) Move up | (↓) Move down | (ENTER) Close Failed Trace";

pub fn ui(f: &mut Frame, mut app: &mut App) {
    match app.local_campaign_data.status {
        CampaignStatus::Initializing => {
            render_initializing(f, app);
        }
        _ => {
            render_running(f, app);
        }
    }
}

fn render_initializing(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10)])
        .split(f.size());

    render_main_widget(f, _app, chunks[0], "Initializing");
}

fn render_running(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.size());

    let status = match app.local_campaign_data.status {
        CampaignStatus::Initializing => "Initializing",
        CampaignStatus::InProgress => "In Progress",
        CampaignStatus::Optimizing => "Optimizing",
        CampaignStatus::Finished => "Finished",
    };
    render_main_widget(f, app, chunks[0], status);

    render_table(f, app, chunks[1]);

    render_scrollbar(f, app, chunks[1]);

    render_footer(f, app, chunks[2]);

    if app.show_popup {
        render_popup(f, app, chunks[1]);
    }
}

fn render_finished(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(5)])
        .split(f.size());

    render_main_widget(f, app, chunks[0], "Finished");

    // let trace_widget = get_trace_widget(app);
    // f.render_widget(trace_widget, chunks[1]);
}

fn render_main_widget(f: &mut Frame, app: &App, area: Rect, status: &str) {
    let title = Title::from(" Inkscope Fuzzer ".bold());
    let status = Title::from(vec![" ".into(), status.into(), " ".into()]);

    let main_block = Block::bordered()
        .title(title.alignment(Alignment::Center))
        .title(status.alignment(Alignment::Center).position(Position::Top))
        .border_type(BorderType::Double)
        .border_style(Style::new().fg(app.colors.footer_border_color));

    let mut lines = vec![];

    let seed_line = Line::from(vec![
        "Seed: ".into(),
        app.local_campaign_data.config.seed.to_string().yellow(),
    ]);
    lines.push(seed_line);

    let n_properties_line = Line::from(vec![
        "Properties found: ".into(),
        app.local_campaign_data
            .properties_or_messages
            .len()
            .to_string()
            .yellow(),
    ]);
    lines.push(n_properties_line);

    let iterations = Line::from(vec![
        Span::styled("Iterations: ", Style::default()),
        Span::styled(
            app.local_campaign_data.current_iteration.to_string(),
            Style::default(),
        )
        .yellow(),
        Span::styled("/", Style::default()).yellow(),
        Span::styled(
            app.local_campaign_data.config.max_rounds.to_string(),
            Style::default(),
        )
        .yellow(),
    ]);
    lines.push(iterations);

    let fail_fast = Line::from(vec![
        "Fail fast: ".into(),
        app.local_campaign_data
            .config
            .fail_fast
            .to_string()
            .yellow(),
    ]);
    lines.push(fail_fast);

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .left_aligned()
        .block(main_block);
    f.render_widget(paragraph, area);
}

fn get_trace_widget(app: &App, method_id: [u8; 4]) -> Paragraph {
    let title = Title::from(" Failed Trace ");
    let trace_block = Block::default()
        .title(title.alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_set(border::PLAIN)
        .style(Style::default());

    let failed_trace = app.local_campaign_data.failed_traces.get(&method_id);
    match failed_trace {
        None => {
            let text = Text::from(vec![Line::from("No failed trace found")]);
            return Paragraph::new(text).centered().block(trace_block);
        }
        Some(failed_trace) => {
            let mut lines = vec![];

            // Messages
            for (idx, deploy_or_message) in failed_trace.trace.messages.iter().enumerate()
            {
                let mut message_data = vec![];
                message_data.push(Span::styled(
                    format!("Message {}: ", idx),
                    Style::default().fg(Color::Yellow),
                ));
                // lines.push(Line::from(format!("Message {}: ", idx)));
                let decode_result = match deploy_or_message {
                    DeployOrMessage::Deploy(deploy) => {
                        app.contract.decode_deploy(&deploy.data)
                    }
                    DeployOrMessage::Message(message) => {
                        app.contract.decode_message(&message.input)
                    }
                };
                match decode_result {
                    Err(_e) => {
                        // lines.push(Line::from(format!("Raw message: {:?}",
                        // &deploy_or_message.data())));
                        message_data.push(Span::styled(
                            format!("Raw message: {:?}", &deploy_or_message.data()),
                            Style::default(),
                        ));
                    }
                    Result::Ok(x) => {
                        // lines.push(Line::from(value_to_string(&x)));
                        message_data
                            .push(Span::styled(value_to_string(&x), Style::default()));
                    }
                }
                lines.push(Line::from(message_data));
            }

            match &failed_trace.reason {
                FailReason::Trapped => {
                    lines.push(Line::from("Last message in trace has Trapped"))
                }
                FailReason::Property(failed_property) => {
                    // Failed properties
                    match app.contract.decode_message(&failed_property.input) {
                        Err(_e) => {
                            lines.push(Line::from(format!(
                                "Raw message: {:?}",
                                &failed_property.input
                            )));
                        }
                        Result::Ok(x) => {
                            lines.push(Line::from(vec![
                                "Property: ".into(),
                                value_to_string(&x).into(),
                            ]));
                        }
                    }
                }
            };
            app.popup_scroll_state.content_length(lines.len());
            Paragraph::new(lines).left_aligned().block(trace_block)
        }
    }
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ["Name", "Type", "Status"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let rows = app
        .local_campaign_data
        .properties_or_messages
        .iter()
        .enumerate()
        .map(|(i, (method_id, method_info))| {
            let color = match i % 2 {
                0 => app.colors.normal_row_color,
                _ => app.colors.alt_row_color,
            };

            let table_type = if method_info.property {
                "Property"
            } else {
                "Message"
            };

            let fuzzing_status =
                match app.local_campaign_data.failed_traces.get(method_id) {
                    Some(_) => {
                        match app.local_campaign_data.status {
                            CampaignStatus::InProgress => "❌ Failed",
                            CampaignStatus::Optimizing => "🔍 Optimizing...",
                            CampaignStatus::Finished => "❌ Failed - Optimized",
                            _ => "",
                        }
                    }
                    None => {
                        match app.local_campaign_data.status {
                            CampaignStatus::InProgress => "🔍 Fuzzing...",
                            CampaignStatus::Finished | CampaignStatus::Optimizing => {
                                "✅ Passed"
                            }
                            _ => "",
                        }
                    }
                };

            vec![
                method_info.method_name.clone(),
                table_type.into(),
                fuzzing_status.into(),
            ]
            .into_iter()
            .map(|content| Cell::from(Text::from(format!("{content}"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(ITEM_HEIGHT as u16)
        });
    let bar = " █ ";
    let t = Table::new(
        rows,
        [
            // + 1 is for padding.
            // Constraint::Length(app.longest_item_lens.0 + 1),
            // Constraint::Min(app.longest_item_lens.1 + 1),
            // Constraint::Min(app.longest_item_lens.2),
            Constraint::Length(20),
            Constraint::Min(5),
            Constraint::Min(5),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(t, area, &mut app.table_state);
}

// fn constraint_len_calculator(items: &[Data]) -> (u16, u16, u16) {
//     let name_len = items
//         .iter()
//         .map(Data::name)
//         .map(UnicodeWidthStr::width)
//         .max()
//         .unwrap_or(0);
//     let address_len = items
//         .iter()
//         .map(Data::address)
//         .flat_map(str::lines)
//         .map(UnicodeWidthStr::width)
//         .max()
//         .unwrap_or(0);
//     let email_len = items
//         .iter()
//         .map(Data::email)
//         .map(UnicodeWidthStr::width)
//         .max()
//         .unwrap_or(0);

//     #[allow(clippy::cast_possible_truncation)]
//     (name_len as u16, address_len as u16, email_len as u16)
// }

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.table_scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.show_popup {
        INFO_TEXT_CLOSE_MODAL
    } else {
        INFO_TEXT_OPEN_MODAL
    };
    let info_footer = Paragraph::new(Line::from(text))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(app.colors.footer_border_color)),
        );
    f.render_widget(info_footer, area);
}

fn render_popup(f: &mut Frame, app: &mut App, area: Rect) {
    let key_index = app.table_state.selected().unwrap();
    let (method_id, method_info) = app
        .local_campaign_data
        .properties_or_messages
        .get(key_index)
        .unwrap();
    let popup_layout = centered_rect(100, 100, area);
    let popup_block = Block::default()
        .title(Title::from(vec![
            Span::styled(" Failed Trace: ", Style::default().fg(app.colors.header_fg)),
            Span::styled(
                format!("{} ", method_info.method_name),
                Style::default().fg(app.colors.row_fg),
            ),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(app.colors.footer_border_color));
    let paragraph = get_trace_widget(app, method_id.to_owned())
        .style(
            Style::default()
                .fg(app.colors.row_fg)
                .bg(app.colors.buffer_bg),
        )
        .wrap(Wrap { trim: true })
        .block(popup_block)
        .scroll((app.popup_scroll_position as u16, 0));

    f.render_widget(Clear, popup_layout);
    f.render_widget(paragraph, popup_layout);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        popup_layout.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.popup_scroll_state,
    );
}

/// helper function to create a centered rect using up certain percentage of the available
/// rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Map(map) => {
            let mut result = String::from(format!("{}(", map.ident().unwrap()));
            for (n, (_, value)) in map.iter().enumerate() {
                if n != 0 {
                    result.push_str(", ");
                }
                result.push_str(&value_to_string(value));
            }
            result.push(')');
            result
        }
        _ => {
            format!("{:?}", value)
        }
    }
}
