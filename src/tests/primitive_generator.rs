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
        let fuzzer: RuntimeFuzzer = RuntimeFuzzer::new(PathBuf::from(
            "./test-contracts/primitive_generator_tester/target/ink/primitive_generator_tester.contract",
        ));
        let mut session = Session::<MinimalRuntime>::new().expect("This should not fail");
        fuzzer.initialize_state(&mut session, &[]);

        // Deploy the contract
        let constructor = fuzzer.generate_constructor();

        let res = session.sandbox().deploy_contract(
            constructor.contract_bytes,
            constructor.endowment,
            constructor.data,
            constructor.salt,
            constructor.caller,
            Weight::max_value() / 10,
            None,
        );
        let contract_address = res.result.expect("Deployment failed").account_id;

        // Generate a bool message
        let message = fuzzer.generate_message(&contract_address, Some("bool_message"));
        let call_result = session.sandbox().call_contract(
            message.callee,
            message.endowment,
            message.input,
            message.caller,
            Weight::max_value() / 10,
            None,
            Determinism::Enforced,
        );

        assert!(call_result.result.is_ok());
        let revert_flag = !call_result.result.unwrap().flags.is_empty();
        assert!(!revert_flag);
    }
}
