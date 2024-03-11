#[cfg(test)]
pub mod primitive_generator_tests {

    use crate::RuntimeFuzzer;
    use std::path::PathBuf;

    #[test]
    fn fuzz_flip() {
        // Initialize the logger
        env_logger::init();

        // Initialize the fuzzer
        let mut fuzzer: RuntimeFuzzer = RuntimeFuzzer::new(PathBuf::from(
            "./test-contracts/flipper/target/ink/flipper.contract",
        ));

        // Run the fuzzer and try to break the defined properties
        let r = fuzzer.run(None);
        assert!(r.is_err());
        let error = r.unwrap_err();
        assert_eq!(error.to_string(), "Property check failed");
    }
}
