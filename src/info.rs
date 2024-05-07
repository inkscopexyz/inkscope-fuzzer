use contract_transcode::Value;

use crate::{
    contract_bundle::ContractBundle,
    engine::{
        CampaignStatus,
        DeployOrMessage,
        FailReason,
        FailedTrace,
    },
    tui::{
        self,
        app::App,
    },
};
use std::{
    io::{
        self,
        Write,
    },
    sync::{
        atomic::AtomicU64,
        Arc,
        RwLock,
    },
    thread::{
        self,
        JoinHandle,
    },
};

use crate::engine::CampaignData;

pub trait OutputTrait {
    fn new(contract: ContractBundle) -> Self;
    fn start_campaign(&mut self, seed: u64, properties: Vec<String>, max_iterations: u64);
    fn end_campaign(&mut self) -> io::Result<()>;
    fn exit(&self) -> bool;
    fn update_status(&mut self, campaign_status: CampaignStatus);
    fn update_failed_traces(&mut self, failed_traces: Vec<FailedTrace>);
    fn incr_iteration(&mut self);
}

pub struct ConsoleOutput {
    pub contract: ContractBundle,
    pub failed_traces: Vec<FailedTrace>,
    pub current_iteration: u64,
}
impl OutputTrait for ConsoleOutput {
    fn new(contract: ContractBundle) -> Self {
        Self {
            contract,
            failed_traces: Vec::new(),
            current_iteration: 0,
        }
    }
    fn start_campaign(
        &mut self,
        seed: u64,
        properties: Vec<String>,
        max_iterations: u64,
    ) {
        println!("Starting campaign...");
        println!("Seed: {}", seed);
        println!("Properties found: {:?}", properties.len());
        println!("Max iterations: {}", max_iterations);
    }
    fn end_campaign(&mut self) -> io::Result<()> {
        if self.failed_traces.is_empty() {
            println!("\nNo bugs found! ✅");
        } else {
            println!("\nSome properties failed! ❌");
        }
        println!("Ending campaign");
        Ok(())
    }
    fn exit(&self) -> bool {
        false
    }
    fn update_status(&mut self, campaign_status: CampaignStatus) {
        println!("Campaign status: {:?}", campaign_status);
    }
    fn update_failed_traces(&mut self, failed_traces: Vec<FailedTrace>) {
        if self.failed_traces.len() < failed_traces.len() {
            self.failed_traces.extend(failed_traces);

            let failed_trace = self.failed_traces.last().unwrap();

            match &failed_trace.reason {
                FailReason::Trapped => {
                    println!(
                        "\nLast message in trace has Trapped or assertion has failed ❌"
                    );
                }
                FailReason::Property(_failed_property) => {
                    println!("\nProperty check failed ❌");
                }
            }

            // Messages
            for (idx, deploy_or_message) in failed_trace.trace.messages.iter().enumerate()
            {
                print!("  Message{}: ", idx);
                let decode_result = match deploy_or_message {
                    DeployOrMessage::Deploy(deploy) => {
                        self.contract.decode_deploy(&deploy.data)
                    }
                    DeployOrMessage::Message(message) => {
                        self.contract.decode_message(&message.input)
                    }
                };
                match decode_result {
                    Err(_e) => {
                        println!("Raw message: {:?}", &deploy_or_message.data());
                    }
                    Result::Ok(x) => {
                        print_value(&x);
                        println!();
                    }
                }
            }

            match &failed_trace.reason {
                FailReason::Trapped => println!("Last message in trace has Trapped"),
                FailReason::Property(failed_property) => {
                    // Failed properties

                    match self.contract.decode_message(&failed_property.input) {
                        Err(_e) => {
                            println!("Raw message: {:?}", &failed_property.input);
                        }
                        Result::Ok(x) => {
                            print!("  Property: ",);
                            print_value(&x);
                            println!();
                        }
                    }
                }
            };
        }
    }
    fn incr_iteration(&mut self) {
        self.current_iteration += 1;
        if self.current_iteration % 10 == 0 {
            print!(".");
            io::stdout().flush().unwrap();
        }
    }
}

pub struct TuiOutput {
    pub campaign_data: Arc<RwLock<CampaignData>>,
    pub contract: ContractBundle,
    pub tui_thread: Option<JoinHandle<()>>,
    pub current_iteration: AtomicU64,
}
impl OutputTrait for TuiOutput {
    fn new(contract: ContractBundle) -> Self {
        let campaign_data = Arc::new(RwLock::new(CampaignData::default()));
        Self {
            campaign_data,
            contract,
            tui_thread: None,
            current_iteration: 0.into(),
        }
    }
    fn start_campaign(
        &mut self,
        seed: u64,
        properties: Vec<String>,
        max_iterations: u64,
    ) {
        self.campaign_data.write().unwrap().seed = seed;
        self.campaign_data.write().unwrap().properties = properties;
        // self.campaign_data.write().unwrap().max_iterations = max_iterations;
        self.campaign_data.write().unwrap().status = CampaignStatus::InProgress;
        let campaign_data = Arc::clone(&self.campaign_data);
        let tui_thread = thread::spawn(move || {
            let mut app = App::new(campaign_data);
            let mut terminal = tui::terminal::init().unwrap();
            app.run(&mut terminal).unwrap();
        });
        self.tui_thread = Some(tui_thread);
    }
    fn end_campaign(&mut self) -> io::Result<()> {
        if let Some(handle) = self.tui_thread.take() {
            // Wait for the tui thread to finish
            handle.join().unwrap();

            // Restore the terminal to its original state
            tui::terminal::restore()?;
        }
        Ok(())
    }
    fn exit(&self) -> bool {
        false
    }
    fn update_status(&mut self, campaign_status: CampaignStatus) {
        self.campaign_data.write().unwrap().status = campaign_status;
    }
    fn update_failed_traces(&mut self, failed_traces: Vec<FailedTrace>) {
        self.campaign_data.write().unwrap().failed_traces = failed_traces;
    }
    fn incr_iteration(&mut self) {
        if self
            .current_iteration
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % 100
            == 0
        {
            println!(".");
        }
    }
}

fn print_value(value: &Value) {
    match value {
        Value::Map(map) => {
            print!("{}(", map.ident().unwrap());
            for (n, (_name, value)) in map.iter().enumerate() {
                if n != 0 {
                    print!(", ");
                }
                print_value(value);
            }
            print!(")");
        }
        _ => {
            print!("{:?}", value);
        }
    }
}
