use crate::tui::{
    self,
    app::App,
};
use anyhow::Result;
use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

use crate::engine::CampaignData;

pub struct Info {
    pub tui_thread: Option<JoinHandle<()>>,
}
impl Info {
    pub fn new() -> Self {
        Self { tui_thread: None }
    }
    pub fn init(
        &mut self,
        campaign_data: Arc<RwLock<CampaignData>>,
        use_tui: bool,
    ) -> Result<()> {
        if use_tui {
            // Initialize the terminal UI
            let mut terminal = tui::terminal::init()?;

            // Initialize the tui app
            let mut app = App::new(campaign_data);

            // Run the tui in a new thread
            self.tui_thread = Some(std::thread::spawn(move || {
                app.run(&mut terminal).unwrap();
            }));
        }

        Ok(())
    }
    pub fn finalize(&mut self) -> Result<()> {
        if let Some(handle) = self.tui_thread.take() {
            // Wait for the tui thread to finish
            handle.join().unwrap();

            // Restore the terminal to its original state
            tui::terminal::restore()?;
        }
        Ok(())
    }
}
