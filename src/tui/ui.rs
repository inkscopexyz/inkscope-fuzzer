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
        Borders,
        Clear,
        List,
        ListItem,
        Paragraph,
        Wrap,
    },
    Frame,
};

use crate::engine::CampaignStatus;

use super::app::App;

// ANCHOR: method_sig
pub fn ui(f: &mut Frame, app: &App) {
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

fn render_initializing(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5)])
        .split(f.size());

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
    let status = Line::from(vec!["Status: Initializing...".into()]);

    let widget = Paragraph::new(status).centered().block(title_block);

    f.render_widget(widget, chunks[0]);
}

fn render_running(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(f.size());

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

    let text = Text::from(lines);

    let a = Paragraph::new(text).centered().block(title_block);

    // let title = Paragraph::new(Text::styled(
    //     "Create New Json",
    //     Style::default().fg(Color::Green),
    // ))
    // .block(title_block);
    f.render_widget(a, chunks[0]);

    let fuzzer_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    f.render_widget(fuzzer_block, chunks[1]);
}

fn render_finished(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(f.size());

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
    let status = Line::from(vec!["Status: Finished...".into()]);

    let widget = Paragraph::new(status).centered().block(title_block);

    f.render_widget(widget, chunks[0]);

    let fuzzer_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let lines;
    if !app.local_campaign_data.failed_traces.is_empty() {
        lines = vec![Line::from("Bug found").alignment(Alignment::Center).red()];
        // Only show the first failed trace
        let failed_trace = &app.local_campaign_data.failed_traces[0];
    } else {
        lines = vec![Line::from("No bug found")
            .alignment(Alignment::Center)
            .green()];
    }
    let widget = Paragraph::new(lines).centered().block(fuzzer_block);
    f.render_widget(widget, chunks[1]);
}
