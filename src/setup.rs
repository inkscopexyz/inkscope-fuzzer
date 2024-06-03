use contract_metadata::ContractMetadata;
use ink_sandbox::{
    api::contracts_api::ContractAPI,
    pallet_contracts::Determinism,
    AccountId32,
    DefaultSandbox,
};
use ratatui::symbols::bar::Set;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use crate::contract_bundle::ContractBundle;

#[derive(Debug, Deserialize)]
struct ConstructorData {
    name: String,
    args: Vec<serde_json::Value>,
    value: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ContractInfo {
    pub contract_path: PathBuf,
    pub constructor_data: Option<Vec<ConstructorData>>,
}

#[derive(Debug, Deserialize)]
pub struct Setup {
    pub network_url: String,
    pub contracts: Vec<ContractInfo>,
}
impl Setup {
    pub fn new(contracts: &PathBuf) -> Self {
        let file = File::open(contracts).expect("Failed to open contracts file");
        let reader = BufReader::new(file);
        let setup_info: Setup =
            serde_json::from_reader(reader).expect("Failed to parse contracts file");
        setup_info
    }

    pub fn deploy_contracts(&self, sandbox: &mut DefaultSandbox, deployer: &AccountId32) {
        self.contracts.iter().for_each(|contract_info| {
            println!("Deploying contract: {:?}", contract_info.contract_path);
            let metadata: ContractMetadata =
                ContractMetadata::load(contract_info.contract_path.clone())
                    .map_err(|e| {
                        anyhow::format_err!("Failed to load the contract file:\n{e:?}")
                    })
                    .unwrap();

            let contract_wasm = metadata.source.wasm.unwrap().0;
            let res = sandbox
                .upload_contract(
                    contract_wasm,
                    deployer.to_owned(),
                    None,
                    Determinism::Enforced,
                )
                .expect("Failed to upload contract");
            println!("Contract deployed at: {:?}", res);
        });

        // let contract_bundle = ContractBundle::load(contracts)?;
        // // Deploy the contract
        // let contract_address = session.deploy_bundle(&contract_bundle)?;
        // println!("Contract deployed at: {:?}", contract_address);
    }
}
