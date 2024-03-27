#[cfg(test)]
pub mod ityfuzz {

    use crate::{config::Config, constants::Constants, engine::Engine};
    use std::{default, path::PathBuf};

    fn test_contract(contract_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 1000,
            max_number_of_transactions: 50, 
            ..Default::default()
        };

        let mut engine = Engine::new(contract_path, config)?;
        let campaign_result = engine.run_campaign()?;
        engine.print_campaign_result(&campaign_result);

        // Check that the campaign result found at least one failing trace
        assert!(!campaign_result.failed_traces.is_empty());

        Ok(())
    }

    #[test]
    fn fuzz_ityfuzz() -> Result<(), Box<dyn std::error::Error>> {
        test_contract(PathBuf::from(
            "./test-contracts/ityfuzz/target/ink/ityfuzz.contract",
        ))
    }
}
