use std::{io::{self, stdout, Stdout}, sync::{Arc, RwLock}, thread, time::Duration};

use crossterm::{execute, terminal::*};
use ratatui::{prelude::*, widgets::{block::Title, Block}};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use crate::engine::CampaignData;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub struct App {
    pub campaign_data: Arc<RwLock<CampaignData>>,
    pub local_campaign_data: CampaignData,
    pub exit: bool,
}
impl App {
    pub fn new(campaign_data: Arc<RwLock<CampaignData>>) -> Self {
        let local_campaign_data = campaign_data.read().unwrap().clone();
        Self {
            campaign_data,
            local_campaign_data,
            exit: false,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit && self.local_campaign_data.in_progress{
            self.local_campaign_data = self.campaign_data.read().unwrap().clone();
            terminal.draw(|frame| self.render_frame(frame))?;
            //self.handle_events()?;
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        todo!()
    }
}
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Inkscope Fuzzer ".bold());
        let instructions = Title::from(Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let mut lines = vec![];
        let seed_line = Line::from(vec![
            "Seed: ".into(),
            self.local_campaign_data.seed.to_string().yellow(),
        ]);
        lines.push(seed_line);

        let n_properties_line = Line::from(vec![
            "Properties N : ".into(),
            self.local_campaign_data.properties.len().to_string().yellow(),
        ]);
        lines.push(n_properties_line);

        let text = Text::from(lines);

        Paragraph::new(text)
            .centered()
            .block(block.clone())
            .render(area, buf);
    }
}