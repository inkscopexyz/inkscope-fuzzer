pub mod app;
pub mod terminal;
pub mod ui;

use crate::{
    config::Config,
    contract_bundle::ContractBundle,
    engine::{
        CampaignStatus,
        FailedTrace,
        MethodInfo,
    },
};
use std::{
    io::{
        self,
    },
    sync::{
        Arc,
        RwLock,
    },
    thread::{
        self,
        JoinHandle,
    },
};

use crate::engine::CampaignData;

use self::app::App;

use super::OutputTrait;

pub struct TuiOutput {
    pub campaign_data: Arc<RwLock<CampaignData>>,
    pub contract: ContractBundle,
    pub tui_thread: Option<JoinHandle<()>>,
}
impl OutputTrait for TuiOutput {
    fn new(contract: ContractBundle) -> Self {
        let campaign_data = Arc::new(RwLock::new(CampaignData::default()));
        Self {
            campaign_data,
            contract,
            tui_thread: None,
        }
    }
    fn start_campaign(
        &mut self,
        config: Config,
        properties_or_messages: Vec<([u8; 4], MethodInfo)>,
    ) {
        {
            let mut shared_campaign_data = self.campaign_data.write().unwrap();
            shared_campaign_data.config = config;
            shared_campaign_data.properties_or_messages = properties_or_messages;
            shared_campaign_data.status = CampaignStatus::InProgress;
        }
        let campaign_data = Arc::clone(&self.campaign_data);
        let contract = self.contract.clone();
        let tui_thread = thread::spawn(move || {
            let mut app = App::new(campaign_data, contract);
            let mut terminal = terminal::init().unwrap();
            app.run(&mut terminal).unwrap();
        });
        self.tui_thread = Some(tui_thread);
    }
    fn end_campaign(&mut self) -> io::Result<()> {
        // Set the campaign status to finished
        self.campaign_data.write().unwrap().status = CampaignStatus::Finished;

        if let Some(handle) = self.tui_thread.take() {
            // Wait for the tui thread to finish
            handle.join().unwrap();

            // Restore the terminal to its original state
            terminal::restore()?;
        }
        Ok(())
    }
    fn exit(&self) -> bool {
        false
    }
    fn update_status(&mut self, campaign_status: CampaignStatus) {
        self.campaign_data.write().unwrap().status = campaign_status;
    }
    fn update_failed_traces(&mut self, key: [u8; 4], failed_trace: FailedTrace) {
        self.campaign_data
            .write()
            .unwrap()
            .failed_traces
            .insert(key, failed_trace);
    }
    fn incr_iteration(&mut self) {
        self.campaign_data.write().unwrap().current_iteration += 1;
    }
}
