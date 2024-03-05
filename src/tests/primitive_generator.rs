#[cfg(test)]
pub mod primitive_generator_tests {
    use drink::{
        frame_support::sp_runtime::traits::Bounded, pallet_contracts::Determinism,
        session::Session, MinimalRuntime, Weight,
    };

    use crate::RuntimeFuzzer;
    use std::path::PathBuf;

    #[test]
    fn bool_message() {
        // Initialize the fuzzer and the runtime
        // Initialize the fuzzer and the runtime
        env_logger::init();

        let mut fuzzer: RuntimeFuzzer = RuntimeFuzzer::new(PathBuf::from(
            "./test-contracts/primitive_generator_tester/target/ink/primitive_generator_tester.contract",
        ));

        let r = fuzzer.run();
        println!("Result: {:?}", r);
        
    }
}
