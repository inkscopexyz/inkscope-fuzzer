use crate::{contract_bundle::ContractBundle, engine::{CampaignStatus, FailedTrace}, tui::{
    self,
    app::App,
}};
use std::{
    io, sync::{Arc, RwLock}, thread::{self, JoinHandle}
};

use crate::engine::CampaignData;


pub trait OutputTrait {
    fn new(contract: ContractBundle )->Self;
    fn start_campaign(&mut self, seed: u64, properties: Vec<String>, max_iterations: u64);
    fn end_campaign(&mut self)-> io::Result<()>;
    fn exit(&self)->bool;
    fn update_status(&mut self, campaign_status: CampaignStatus);
    fn update_failed_traces(&mut self, failed_traces: Vec<FailedTrace>);
}

pub struct ConsoleOutput{
    pub contract: ContractBundle,
    pub failed_traces: Vec<FailedTrace>,
}
impl OutputTrait for ConsoleOutput{
    fn new(contract: ContractBundle)->Self{
        Self{contract, failed_traces: Vec::new()}
    }
    fn start_campaign(&mut self, seed: u64, properties: Vec<String>, max_iterations: u64){
        println!("Starting campaign...");
        println!("Seed: {}", seed);
        println!("Properties found: {:?}", properties.len());
        println!("Max iterations: {}", max_iterations);
    }
    fn end_campaign(&mut self) -> io::Result<()>{
        println!("Ending campaign");
        Ok(())
    }
    fn exit(&self)->bool{
        false
    }
    fn update_status(&mut self, campaign_status: CampaignStatus){
        println!("Campaign status: {:?}", campaign_status);
    }
    fn update_failed_traces(&mut self, failed_traces: Vec<FailedTrace>){
    }
}

pub struct TuiOutput {
    pub campaign_data: Arc<RwLock<CampaignData>>,
    pub contract: ContractBundle,
    pub tui_thread: Option<JoinHandle<()>>,
}
impl OutputTrait for TuiOutput {
    fn new(contract: ContractBundle )->Self{
        let campaign_data = Arc::new(RwLock::new(CampaignData::default()));
        Self{campaign_data, contract, tui_thread: None}
    }
    fn start_campaign(&mut self, seed: u64, properties: Vec<String>, max_iterations: u64){
        self.campaign_data.write().unwrap().seed = seed;
        self.campaign_data.write().unwrap().properties = properties;
        //self.campaign_data.write().unwrap().max_iterations = max_iterations;
        self.campaign_data.write().unwrap().status = CampaignStatus::InProgress;
        let campaign_data = Arc::clone(&self.campaign_data);
        let tui_thread = thread::spawn(move || {
            let mut app = App::new(campaign_data);
            let mut terminal = tui::terminal::init().unwrap();
            app.run(&mut terminal).unwrap();
        });
        self.tui_thread = Some(tui_thread);
    }
    fn end_campaign(&mut self) -> io::Result<()>{
        if let Some(handle) = self.tui_thread.take() {
            // Wait for the tui thread to finish
            handle.join().unwrap();

            // Restore the terminal to its original state
            tui::terminal::restore()?;
        }
        Ok(())
    }
    fn exit(&self)->bool{
       false
    }
    fn update_status(&mut self, campaign_status: CampaignStatus){
        self.campaign_data.write().unwrap().status = campaign_status;
    }
    fn update_failed_traces(&mut self, failed_traces: Vec<FailedTrace>){
        self.campaign_data.write().unwrap().failed_traces = failed_traces;
    }
}
