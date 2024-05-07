use std::{
    io::{
        self,
        stdout,
        Stdout,
    },
    sync::{
        Arc,
        RwLock,
    },
    thread,
    time::Duration,
};

use ratatui::{
    prelude::*,
    widgets::{
        block::Title,
        Block,
    },
};

use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
};
use ratatui::{
    symbols::border,
    widgets::{
        block::*,
        *,
    },
};

use crate::engine::CampaignData;

use crossterm::{
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use super::ui::ui;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

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
        self.init_terminal()?;
        while !self.exit {
            self.local_campaign_data = self.campaign_data.read().unwrap().clone();
            terminal.draw(|frame| ui(frame, self))?;
            self.handle_events()?;
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_event(key_event);
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // TODO: Add more key bindings here if needed. Handle unwrap
            KeyCode::Char('q') => self.exit().unwrap(),
            _ => {}
        }
    }

    fn exit(&mut self) -> io::Result<()> {
        self.exit = true;
        self.restore_terminal()?;
        Ok(())
    }

    fn init_terminal(&self) -> io::Result<Tui> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    /// Restore the terminal to its original state
    fn restore_terminal(&self) -> io::Result<()> {
        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
