use std::{
    collections::HashMap,
    io::{
        self,
        Write,
    },
};

use crate::{
    config::Config,
    contract_bundle::ContractBundle,
    engine::{
        CampaignStatus,
        DeployOrMessage,
        FailReason,
        FailedTrace,
        MethodInfo,
    },
    output::OutputTrait,
};

pub struct ConsoleOutput {
    pub properties_or_messages: Vec<([u8; 4], MethodInfo)>,
    pub id_to_name: HashMap<[u8; 4], String>,
    pub contract: ContractBundle,
    pub failed_traces: HashMap<[u8; 4], FailedTrace>,
    pub current_iteration: u64,
    pub status: CampaignStatus,
}
impl OutputTrait for ConsoleOutput {
    fn new(contract: ContractBundle) -> Self {
        Self {
            properties_or_messages: Vec::new(),
            id_to_name: HashMap::new(),
            contract,
            failed_traces: HashMap::new(),
            current_iteration: 0,
            status: CampaignStatus::Initializing,
        }
    }
    fn start_campaign(
        &mut self,
        config: Config,
        properties_or_messages: Vec<([u8; 4], MethodInfo)>,
    ) {
        self.status = CampaignStatus::InProgress;
        self.properties_or_messages.clone_from(&properties_or_messages);
        for (id, method_info) in &self.properties_or_messages {
            self.id_to_name.insert(*id, method_info.method_name.clone());
        }

        println!("Starting campaign...");
        println!("Seed: {}", config.seed);
        println!("Properties found: {:?}", properties_or_messages.len());
        println!("Max iterations: {}", config.max_rounds);
        println!("Fail fast: {}", config.fail_fast);
    }

    fn end_campaign(&mut self) -> io::Result<()> {
        for (method_id, _) in self.properties_or_messages.clone() {
            let failed_trace = self.failed_traces.get(&method_id);
            match failed_trace {
                Some(failed_trace) => {
                    println!(
                        "Property {} failed ❌",
                        self.id_to_name
                            .get(&method_id)
                            .unwrap_or(&"Unknown".to_string())
                    );
                    ////
                    // Messages
                    for (idx, deploy_or_message) in
                        failed_trace.trace.messages.iter().enumerate()
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
                        FailReason::Trapped => {
                            println!("Last message in trace has Trapped")
                        }
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
                _ => {
                    println!(
                        "Property {} passed ✅",
                        self.id_to_name
                            .get(&method_id)
                            .unwrap_or(&"Unknown".to_string())
                    );
                }
            }
        }
        self.status = CampaignStatus::Finished;
        println!("Campaign finished");
        Ok(())
    }

    fn exit(&self) -> bool {
        false
    }

    fn update_status(&mut self, campaign_status: CampaignStatus) {
        println!("\nCampaign status: {:?}", &campaign_status);
        self.status = campaign_status;
    }

    fn update_failed_traces(&mut self, key: [u8; 4], new_failed_trace: FailedTrace) {
        let old_failed_trace = self.failed_traces.get(&key);
        match old_failed_trace {
            Some(old_failed_trace) if new_failed_trace >= *old_failed_trace => {
                return;
            }
            None if matches!(self.status, CampaignStatus::InProgress) => {
                match &new_failed_trace.reason {
                    FailReason::Trapped => {
                        print!("❗️");
                    }
                    FailReason::Property(_) => {
                        print!("❌");
                    }
                }
            }
            _ => {}
        }
        self.failed_traces.insert(key, new_failed_trace);
    }
    fn incr_iteration(&mut self) {
        self.current_iteration += 1;
        if self.current_iteration % 10 == 0 {
            print!(".");
            io::stdout().flush().unwrap();
        }
    }
}

use contract_transcode::Value;
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
