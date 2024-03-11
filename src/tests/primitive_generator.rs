#[cfg(test)]
pub mod primitive_generator_tests {
    use drink::{
        frame_support::sp_runtime::traits::Bounded, pallet_contracts::Determinism,
        session::Session, MinimalRuntime, Weight,
    };

    use crate::RuntimeFuzzer;
    use std::path::PathBuf;

    struct PTest {
        path: PathBuf,
        max_rounds: usize,
    }

    #[test]
    fn bool_message() {
        // TODO make ia fixture
        // let all_tests = vec![
        //     PTest{ path: "./test-contracts/primitive_generator_tester/target/ink/primitive_generator_tester.contract",
        //            max_rounds: 100
        //         },
        // ];

        // Initialize the fuzzer and the runtime
        env_logger::init();

        let path = "./test-contracts/primitive_generator_tester/target/ink/primitive_generator_tester.contract";

        let mut fuzzer: RuntimeFuzzer = RuntimeFuzzer::new(PathBuf::from(path));

        let r = fuzzer.run(None);
        println!("Result: {:?}", r);
    }
}
