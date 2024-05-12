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
        }, Block, BorderType, Borders, Cell, Clear, HighlightSpacing, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table, Wrap
    },
    Frame,
};

use crate::engine::{
    CampaignStatus,
    DeployOrMessage,
    FailReason,
};

use super::app::App;

const INFO_TEXT: &str =
    "(Q) quit | (↑) move up | (↓) move down | (→) next color | (←) previous color";

pub fn ui(f: &mut Frame, mut app: &mut App) {
    match app.local_campaign_data.status {
        CampaignStatus::Initializing => {
            render_initializing(f, app);
        }
        CampaignStatus::InProgress => {
            render_running(f, app);
        }
        CampaignStatus::Finished => {
            render_finished(f, app);
        }
    }
}

fn render_initializing(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5)])
        .split(f.size());

    let main_widget = get_main_widget("Initializing");
    f.render_widget(main_widget, chunks[0]);
}

fn render_running(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.size());

    let main_widget = get_main_widget("In Progress");
    f.render_widget(main_widget, chunks[0]);

    let config_widget = get_config_widget(app);
    f.render_widget(config_widget, chunks[1]);

    //let trace_widget = get_trace_widget(app);
    //f.render_widget(trace_widget, chunks[2]);

    render_table(f, app, chunks[2]);

    render_footer(f, app, chunks[3]);
}

fn render_finished(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(5),
        ])
        .split(f.size());

    let main_widget = get_main_widget("Finished");
    f.render_widget(main_widget, chunks[0]);

    let config_widget = get_config_widget(app);
    f.render_widget(config_widget, chunks[1]);

    let trace_widget = get_trace_widget(app);
    f.render_widget(trace_widget, chunks[2]);
}

fn get_main_widget(status: &str) -> Paragraph {
    let title = Title::from(" Inkscope Fuzzer ".bold());
    let instructions =
        Title::from(Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]));
    let title_block = Block::default()
        .title(title.alignment(Alignment::Center))
        .title(
            instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        )
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .style(Style::default());

    // let status = Line::from(vec!["Status: Initializing...".into()]);
    let status = vec![
        Line::from(""),
        Line::from(vec!["Status: ".into(), status.into()]),
        Line::from(""),
    ];

    Paragraph::new(status).centered().block(title_block)
}

fn get_config_widget(app: &App) -> Paragraph {
    let title = Title::from(" Config ");

    let title_block = Block::default()
        .title(title.alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_set(border::PLAIN)
        .style(Style::default());

    let mut lines = vec![];
    let seed_line = Line::from(vec![
        "Seed: ".into(),
        app.local_campaign_data.seed.to_string().yellow(),
    ]);
    lines.push(seed_line);

    let n_properties_line = Line::from(vec![
        "Properties found: ".into(),
        app.local_campaign_data
            .properties
            .len()
            .to_string()
            .yellow(),
    ]);
    lines.push(n_properties_line);

    let properties_line = Line::from(vec![
        "Properties: ".into(),
        app.local_campaign_data.properties.join(", ").yellow(),
    ]);
    lines.push(properties_line);

    let text = Text::from(lines);

    Paragraph::new(text).left_aligned().block(title_block)
}

fn get_trace_widget(app: &App) -> Paragraph {
    let title = Title::from(" Failed Traces ");
    let trace_block = Block::default()
        .title(title.alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_set(border::PLAIN)
        .style(Style::default());

    if app.local_campaign_data.failed_traces.is_empty() {
        let text = Text::from(vec![Line::from("No failed traces found")]);
        return Paragraph::new(text).centered().block(trace_block);
    } else {
        let mut lines = vec![];
        for failed_trace in &app.local_campaign_data.failed_traces {
            match &failed_trace.reason {
                FailReason::Trapped => {
                    lines.push(Line::from(
                        "Last message in trace has Trapped or assertion has failed ❌",
                    ));
                }
                FailReason::Property(_failed_property) => {
                    lines.push(Line::from("Property check failed ❌"));
                }
            }

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

            lines.push(Line::from(""));
        }
        Paragraph::new(lines).left_aligned().block(trace_block)
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
    let rows = app.local_campaign_data.properties.iter().enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        // let item = data.ref_array();
        // item.into_iter()
        //     .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
        //     .collect::<Row>()
        //     .style(Style::new().fg(app.colors.row_fg).bg(color))
        //     .height(4)
        //Row::from
        
        vec![
            data,
            data,
            data
        ].into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(4)
        
        
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
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(INFO_TEXT))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(app.colors.footer_border_color)),
        );
    f.render_widget(info_footer, area);
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
