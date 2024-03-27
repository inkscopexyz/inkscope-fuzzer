use anyhow::Result;
use drink::{
    frame_support::sp_runtime::traits::Bounded,
    Weight,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs::File,
    path::Path,
};

use crate::{
    constants::Constants,
    types::{
        AccountId,
        Balance,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Max endowment that can be sent in a fuzzed message
    pub budget: Balance,

    // Accounts that can be used as caller
    pub accounts: Vec<AccountId>,

    // If true, only mutable messages will be fuzzed
    pub only_mutable: bool,

    // Max length when genrating fuzzed values fot arbitrary long sequence types
    pub max_sequence_type_size: u8,

    // Max number of transactions that can be generated in a given run
    pub max_number_of_transactions: usize,

    // Max gas limit for a transaction
    pub gas_limit: Weight,

    // Prefix for the property name. This is used to identify the property from normal
    // messages
    pub property_prefix: String,

    // In case the property takes arguments, how many rounds to spend fuzzing the
    // arguments
    pub fuzz_property_max_rounds: usize,

    // Initial set of constant values to use in the fuzzing
    pub constants: Constants,
}
impl Config {
    pub fn new(
        budget: Balance,
        accounts: Vec<AccountId>,
        only_mutable: bool,
        max_sequence_type_size: u8,
        max_number_of_transactions: usize,
        gas_limit: Weight,
        property_prefix: String,
        fuzz_property_max_rounds: usize,
        constants: Constants,
    ) -> Self {
        Self {
            budget,
            accounts,
            only_mutable,
            max_sequence_type_size,
            max_number_of_transactions,
            gas_limit,
            property_prefix,
            fuzz_property_max_rounds,
            constants,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        // TODO! Do it right
        // let default_callers: Vec<AccountId> = vec![
        //     "Alice".into(),
        //     "Bob".into(),
        //     "Charlie".into(),
        //     "Dave".into(),
        //     "Eve".into(),
        //     "Ferdinand".into(),
        //     "Gina".into(),
        //     "Hank".into(),
        //     "Ivan".into(),
        //     "Jenny".into(),
        // ];

        Self {
            budget: 100,
            accounts: vec![AccountId::new([0; 32]), AccountId::new([1; 32])],
            only_mutable: true,
            max_sequence_type_size: 10,
            max_number_of_transactions: 10,
            gas_limit: Weight::max_value() / 4,
            property_prefix: "inkscope_".to_string(),
            fuzz_property_max_rounds: 100,
            constants: Constants::default(),
        }
    }
}

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let fp = File::open(file)?;
        let des = serde_yaml::from_reader(fp)?;
        Ok(des)
    }
    pub fn to_yaml_file<P: AsRef<Path>>(&self, file: P) -> Result<()> {
        let fp = File::create(file)?;
        serde_yaml::to_writer(fp, self)?;
        Ok(())
    }
}
