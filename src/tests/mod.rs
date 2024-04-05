#[cfg(test)]
pub mod testing {

    use crate::{
        config::Config,
        engine::Engine,
    };
    use std::path::PathBuf;

    fn test_contract(
        contract_path: PathBuf,
        should_find_broken_properties: bool,
        max_rounds: Option<usize>,
        max_number_of_transactions: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: max_rounds.unwrap_or(10),
            max_number_of_transactions: max_number_of_transactions.unwrap_or(50),
            ..Default::default()
        };

        let mut engine = Engine::new(contract_path, config)?;
        let campaign_result = engine.run_campaign()?;
        engine.print_campaign_result(&campaign_result);

        // Check that the campaign result is as expected
        if should_find_broken_properties {
            assert!(!campaign_result.failed_traces.is_empty());
        } else {
            assert!(campaign_result.failed_traces.is_empty());
        }

        Ok(())
    }

    #[test]
    fn fuzz_ityfuzz() -> Result<(), Box<dyn std::error::Error>> {
        test_contract(
            PathBuf::from("./test-contracts/ityfuzz/target/ink/ityfuzz.contract"),
            true,
            Some(1000),
            None,
        )
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_1_vulnerable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-1/vulnerable-example/target/ink/integer_overflow_or_underflow.contract", 
        ),true,Some(100),None)
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_1_remediated(
    ) -> Result<(), Box<dyn std::error::Error>> {
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-1/remediated-example/target/ink/integer_overflow_or_underflow.contract", 
        ),false,Some(10),Some(10))
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_2_vulnerable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-2/vulnerable-example/target/ink/integer_overflow_or_underflow.contract", 
        ),true,Some(100),None)
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_2_remediated(
    ) -> Result<(), Box<dyn std::error::Error>> {
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-2/remediated-example/target/ink/integer_overflow_or_underflow.contract", 
        ),false,Some(10),Some(10))
    }
}
