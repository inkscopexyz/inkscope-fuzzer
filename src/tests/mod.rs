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
        config: Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 1000,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(
            PathBuf::from("./test-contracts/ityfuzz/target/ink/ityfuzz.contract"),
            true,
            config,
        )
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_1_vulnerable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-1/vulnerable-example/target/ink/integer_overflow_or_underflow.contract", 
        ),true,config)
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_1_remediated(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 10,
            max_number_of_transactions: 10,
            ..Default::default()
        };
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-1/remediated-example/target/ink/integer_overflow_or_underflow.contract", 
        ),false,config)
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_2_vulnerable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-2/vulnerable-example/target/ink/integer_overflow_or_underflow.contract", 
        ),true,config)
    }

    #[test]
    fn fuzz_integer_overflow_or_underflow_2_remediated(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 10,
            max_number_of_transactions: 10,
            ..Default::default()
        };
        test_contract(PathBuf::from(
            "./test-contracts/coinfabrik-test-contracts/integer-overflow-or-underflow-2/remediated-example/target/ink/integer_overflow_or_underflow.contract", 
        ),false,config)
    }

    #[test]
    fn fuzz_message_panics() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/message-panics/target/ink/message_panics.contract",
            ),
            true,
            config,
        )
    }

    #[test]
    fn fuzz_constructor_panics() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/constructor-panics/target/ink/constructor_panics.contract",
            ),
            true,
            config,
        )
    }

    #[test]
    fn fuzz_iterators_over_indexing_vulnerable() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            only_mutable: false,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/coinfabrik-test-contracts/iterators-over-indexing/vulnerable-example/target/ink/iterators_over_indexing.contract",
            ),
            true,
            config,
        )
    }

    #[test]
    fn fuzz_iterators_over_indexing_remediated() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            only_mutable: false,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/coinfabrik-test-contracts/iterators-over-indexing/remediated-example/target/ink/iterators_over_indexing.contract",
            ),
            false,
            config,
        )
    }

    #[test]
    fn fuzz_assert_violation_vulnerable() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            only_mutable: false,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/coinfabrik-test-contracts/assert-violation/vulnerable-example/target/ink/assert_violation.contract",
            ),
            true,
            config,
        )
    }

    #[test]
    fn fuzz_assert_violation_remediated() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            only_mutable: false,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/coinfabrik-test-contracts/assert-violation/remediated-example/target/ink/assert_violation.contract",
            ),
            false,
            config,
        )
    }

    #[test]
    fn fuzz_zero_or_test_address_vulnerable() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/coinfabrik-test-contracts/zero-or-test-address/vulnerable-example/target/ink/zerocheck.contract",
            ),
            true,
            config,
        )
    }

    #[test]
    fn fuzz_zero_or_test_address_remediated() -> Result<(), Box<dyn std::error::Error>> {
        // Set up the fuzzer configuration
        let config = Config {
            fail_fast: true,
            max_rounds: 100,
            max_number_of_transactions: 50,
            ..Default::default()
        };
        test_contract(
            PathBuf::from(
                "./test-contracts/coinfabrik-test-contracts/zero-or-test-address/remediated-example/target/ink/zerocheck.contract",
            ),
            false,
            config,
        )
    }
}
