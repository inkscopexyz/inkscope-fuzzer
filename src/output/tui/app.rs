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

use ratatui::prelude::*;

use crossterm::event::{
    self,
    KeyCode,
    KeyEvent,
    KeyEventKind,
};
use ratatui::widgets::*;

use crate::{
    contract_bundle::ContractBundle,
    engine::CampaignData,
};

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

use style::palette::tailwind;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

// const COLOR_PALETTE: tailwind::Palette = tailwind::BLUE;

pub struct AppColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub footer_border_color: Color,
}

impl AppColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

pub const ITEM_HEIGHT: usize = 1;

pub struct App {
    pub table_state: TableState,
    pub table_scroll_state: ScrollbarState,
    pub show_popup: bool,
    pub popup_scroll_state: ScrollbarState,
    pub popup_scroll_position: usize,
    pub colors: AppColors,
    pub campaign_data: Arc<RwLock<CampaignData>>,
    pub contract: ContractBundle,
    pub local_campaign_data: CampaignData,
    pub exit: bool,
}
impl App {
    pub fn new(
        campaign_data: Arc<RwLock<CampaignData>>,
        contract: ContractBundle,
    ) -> Self {
        let local_campaign_data = campaign_data.read().unwrap().clone();
        Self {
            table_state: TableState::default().with_selected(0),
            table_scroll_state: ScrollbarState::new(
                (local_campaign_data.properties_or_messages.len() - 1) * ITEM_HEIGHT,
            ),
            show_popup: false,
            popup_scroll_state: ScrollbarState::new(0),
            popup_scroll_position: 0,
            colors: AppColors::new(&tailwind::BLUE),
            campaign_data,
            contract,
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
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            KeyCode::Enter => self.toggle_popup(),
            KeyCode::Esc if self.show_popup => self.toggle_popup(),
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

    pub fn next(&mut self) {
        if self.show_popup {
            self.popup_scroll_position = self.popup_scroll_position.saturating_add(1);
            self.popup_scroll_state =
                self.popup_scroll_state.position(self.popup_scroll_position);
        } else {
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i >= self.local_campaign_data.properties_or_messages.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.table_state.select(Some(i));
            self.table_scroll_state = self.table_scroll_state.position(i * ITEM_HEIGHT);
        }
    }

    pub fn previous(&mut self) {
        if self.show_popup {
            self.popup_scroll_position = self.popup_scroll_position.saturating_sub(1);
            self.popup_scroll_state =
                self.popup_scroll_state.position(self.popup_scroll_position);
        } else {
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.local_campaign_data.properties_or_messages.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.table_state.select(Some(i));
            self.table_scroll_state = self.table_scroll_state.position(i * ITEM_HEIGHT);
        }
    }

    pub fn toggle_popup(&mut self) {
        if !self.show_popup {
            let key_index = self.table_state.selected().unwrap();
            let (method_id, method_info) = self
                .local_campaign_data
                .properties_or_messages
                .get(key_index)
                .unwrap();

            let failed_trace = self.local_campaign_data.failed_traces.get(method_id);
            if failed_trace.is_some() {
                self.show_popup = true;
            }
        } else {
            self.show_popup = false;
        }
    }
}
