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

use super::app::{App, ITEM_HEIGHT};

const INFO_TEXT_OPEN_MODAL: &str =
    "(Q) Quit | (↑) Move up | (↓) Move down | (ENTER) Open Failed Trace";

const INFO_TEXT_CLOSE_MODAL: &str =
    "(Q) Quit | (↑) Move up | (↓) Move down | (ENTER) Close Failed Trace";


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
        .constraints([Constraint::Length(10)])
        .split(f.size());

    render_main_widget(f, _app, chunks[0], "Initializing");
}

fn render_running(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.size());

    render_main_widget(f, app, chunks[0], "In Progress");

    render_table(f, app, chunks[1]);

    render_scrollbar(f, app, chunks[1]);

    render_footer(f, app, chunks[2]);

    if app.show_popup {
        render_popup(f, app, f.size());
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
    let main_block = Block::bordered()
        .title(title.alignment(Alignment::Center))
        .border_type(BorderType::Double)
        .border_style(Style::new().fg(app.colors.footer_border_color));

    let mut lines = vec![];

    let status_line = Line::from(vec!["Status: ".into(), status.into()]);
    lines.push(status_line);

    let seed_line = Line::from(vec![
        "Seed: ".into(),
        app.local_campaign_data.seed.to_string().yellow(),
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

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .left_aligned()
        .block(main_block);
    f.render_widget(paragraph, area);
}

// fn get_trace_widget(app: &App) -> Paragraph {
//     let title = Title::from(" Failed Traces ");
//     let trace_block = Block::default()
//         .title(title.alignment(Alignment::Left))
//         .borders(Borders::ALL)
//         .border_set(border::PLAIN)
//         .style(Style::default());

//     if app.local_campaign_data.failed_traces.is_empty() {
//         let text = Text::from(vec![Line::from("No failed traces found")]);
//         return Paragraph::new(text).centered().block(trace_block);
//     } else {
//         let mut lines = vec![];
//         for failed_trace in &app.local_campaign_data.failed_traces {
//             match &failed_trace.reason {
//                 FailReason::Trapped => {
//                     lines.push(Line::from(
//                         "Last message in trace has Trapped or assertion has failed ❌",
//                     ));
//                 }
//                 FailReason::Property(_failed_property) => {
//                     lines.push(Line::from("Property check failed ❌"));
//                 }
//             }

//             // Messages
//             for (idx, deploy_or_message) in failed_trace.trace.messages.iter().enumerate()
//             {
//                 let mut message_data = vec![];
//                 message_data.push(Span::styled(
//                     format!("Message {}: ", idx),
//                     Style::default().fg(Color::Yellow),
//                 ));
//                 // lines.push(Line::from(format!("Message {}: ", idx)));
//                 let decode_result = match deploy_or_message {
//                     DeployOrMessage::Deploy(deploy) => {
//                         app.contract.decode_deploy(&deploy.data)
//                     }
//                     DeployOrMessage::Message(message) => {
//                         app.contract.decode_message(&message.input)
//                     }
//                 };
//                 match decode_result {
//                     Err(_e) => {
//                         // lines.push(Line::from(format!("Raw message: {:?}",
//                         // &deploy_or_message.data())));
//                         message_data.push(Span::styled(
//                             format!("Raw message: {:?}", &deploy_or_message.data()),
//                             Style::default(),
//                         ));
//                     }
//                     Result::Ok(x) => {
//                         // lines.push(Line::from(value_to_string(&x)));
//                         message_data
//                             .push(Span::styled(value_to_string(&x), Style::default()));
//                     }
//                 }
//                 lines.push(Line::from(message_data));
//             }

//             match &failed_trace.reason {
//                 FailReason::Trapped => {
//                     lines.push(Line::from("Last message in trace has Trapped"))
//                 }
//                 FailReason::Property(failed_property) => {
//                     // Failed properties
//                     match app.contract.decode_message(&failed_property.input) {
//                         Err(_e) => {
//                             lines.push(Line::from(format!(
//                                 "Raw message: {:?}",
//                                 &failed_property.input
//                             )));
//                         }
//                         Result::Ok(x) => {
//                             lines.push(Line::from(vec![
//                                 "Property: ".into(),
//                                 value_to_string(&x).into(),
//                             ]));
//                         }
//                     }
//                 }
//             };

//             lines.push(Line::from(""));
//         }
//         Paragraph::new(lines).left_aligned().block(trace_block)
//     }
// }

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
            // let item = data.ref_array();
            // item.into_iter()
            //     .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            //     .collect::<Row>()
            //     .style(Style::new().fg(app.colors.row_fg).bg(color))
            //     .height(4)
            // Row::from
            let table_type = if method_info.property {
                "Property"
            } else {
                "Message"
            };

            vec![
                method_info.method_name.clone(),
                table_type.into(),
                String::from("Checking... ⚙️"),
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

fn render_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_layout = centered_rect(60, 20, f.size());
    let popup_block = Block::default()
        .title("Popup")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(app.colors.footer_border_color));

    let paragraph = Paragraph::new("Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur?")
        .style(Style::default().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .wrap(Wrap { trim: true }).block(popup_block);
    f.render_widget(Clear, popup_layout);
    f.render_widget(paragraph, popup_layout);
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
