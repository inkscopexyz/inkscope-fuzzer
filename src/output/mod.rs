pub mod console;
pub mod tui;

use crate::{
    config::Config,
    contract_bundle::ContractBundle,
    engine::{
        CampaignStatus,
        FailedTrace,
        MethodInfo,
    },
};
use std::io;

pub use console::ConsoleOutput;
pub use tui::TuiOutput;

pub trait OutputTrait {
    fn new(contract: ContractBundle) -> Self;
    fn start_campaign(&mut self, config: Config, properties: Vec<([u8; 4], MethodInfo)>);
    fn end_campaign(&mut self) -> io::Result<()>;
    fn exit(&self) -> bool;
    fn update_status(&mut self, campaign_status: CampaignStatus);
    fn update_failed_traces(&mut self, key: [u8; 4], new_failed_trace: FailedTrace);
    fn incr_iteration(&mut self);
}
