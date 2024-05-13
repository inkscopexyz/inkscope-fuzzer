use anyhow::Result;
use ink_sandbox::{
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    /// The initial random seed
    pub seed: u64,

    // Exits as soon as a failed property is found
    pub fail_fast: bool,

    // Max number of iterations to run the fuzzer
    pub max_rounds: u64,

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

    // Max attempts to try optimize (reduce) a failed trace.
    pub max_optimization_rounds: usize,

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

    // If true, the fuzzer will use the TUI
    pub use_tui: bool,
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
            seed: 0,
            fail_fast: true,
            max_rounds: 1000,
            budget: 1000000000000,
            accounts: vec![AccountId::new([1; 32]), AccountId::new([2; 32])],
            only_mutable: true,
            max_sequence_type_size: 10,
            max_number_of_transactions: 50,
            max_optimization_rounds: 50,
            gas_limit: Weight::max_value(),
            property_prefix: "inkscope_".to_string(),
            fuzz_property_max_rounds: 100,
            constants: Constants::default(),
            use_tui: false,
        }
    }
}

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let fp = File::open(file)?;
        let des = serde_yaml::from_reader(fp)?;
        Ok(des)
    }

    #[allow(dead_code)]
    pub fn to_yaml_file<P: AsRef<Path>>(&self, file: P) -> Result<()> {
        let fp = File::create(file)?;
        serde_yaml::to_writer(fp, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config() {
        // Test from and to file in memory
        let config = Config::default();
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test_config.yaml");
        config.to_yaml_file(file.to_str().unwrap()).unwrap();
        let config2 = Config::from_yaml_file(file).unwrap();
        assert_eq!(config, config2);
    }
}
