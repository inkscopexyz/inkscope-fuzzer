use std::{sync::{Arc, RwLock}, thread::JoinHandle};

use crate::{engine::CampaignData, tui::{self, App}};
use anyhow::Result;

pub fn init(campaign_data: Arc<RwLock<CampaignData>>, use_tui: bool) -> Result<Option<JoinHandle<()>>> {
    // Initialize the terminal UI
    let mut terminal = tui::init()?;
    let mut app = App::new(campaign_data);
    // Run the tui in a new thread
    let thread_handle = std::thread::spawn(move || {
        app.run(&mut terminal).unwrap();
    });
    Ok(Some(thread_handle))
}

pub fn finalize(thread_handle: Option<JoinHandle<()>>) -> Result<()> {
    // Restore the terminal to its original state
    if let Some(handle) = thread_handle {
        handle.join().unwrap();
        tui::restore()?;
    }
    Ok(())
}
