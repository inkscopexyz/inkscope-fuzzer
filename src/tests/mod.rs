#[cfg(test)]
pub mod ityfuzz {

    use crate::{
        config::Config,
        engine::Engine,
    };
    use std::path::PathBuf;

    fn test_contract(contract_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config =
            Config::from_yaml_file("./config.yaml").expect("failed to parse yaml file");

        let mut engine = Engine::new(contract_path, config)?;
        let campaign_result = engine.run_campaign(1000, true)?;
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
