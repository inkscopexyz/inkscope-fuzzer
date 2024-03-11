#[cfg(test)]
pub mod primitive_generator_tests {

    use crate::RuntimeFuzzer;
    use std::path::PathBuf;

    #[test]
    #[ignore]
    fn fuzz_ityfuzz() {
        // Initialize the logger
        env_logger::init();

        // Initialize the fuzzer
        let mut fuzzer: RuntimeFuzzer = RuntimeFuzzer::new(PathBuf::from(
            "./test-contracts/ityfuzz/target/ink/ityfuzz.contract",
        ));

        // Run the fuzzer and try to break the defined properties
        let r = fuzzer.run(None);
        assert!(r.is_err());
        let error = r.unwrap_err();
        assert_eq!(error.to_string(), "Property check failed");
    }
}

// #[cfg(test)]
// pub mod ityfuzz_tests {
//     use drink::{
//         frame_support::sp_runtime::traits::Bounded, pallet_contracts::Determinism,
//         session::Session, MinimalRuntime, Weight,
//     };
//     use parity_scale_codec::{Decode, Encode};

//     use crate::RuntimeFuzzer;
//     use std::path::PathBuf;

//     #[test]
//     fn happy_path() {
//         // Initialize the fuzzer and the runtime
//         let fuzzer: RuntimeFuzzer = RuntimeFuzzer::new(PathBuf::from(
//             "./test-contracts/ityfuzz/target/ink/ityfuzz.contract",
//         ));
//         let mut session = Session::<MinimalRuntime>::new().expect("This should not fail");
//         fuzzer.initialize_state(&mut session);

//         // Deploy the contract
//         let constructor = fuzzer.generate_constructor();

//         let res = session.sandbox().deploy_contract(
//             constructor.contract_bytes,
//             constructor.endowment,
//             constructor.data,
//             constructor.salt,
//             constructor.caller,
//             Weight::max_value() / 10,
//             None,
//         );
//         let contract_address = res.result.expect("Deployment failed").account_id;

//         // Generate a bool message
//         let selector = fuzzer
//             .contract
//             .transcoder
//             .metadata()
//             .spec()
//             .messages()
//             .iter()
//             .find(|m| m.label() == "incr")
//             .unwrap()
//             .selector();
//         let selector = selector.to_bytes().to_vec();
//         let encoded_args = 0u128.encode();

//         // Concatenate the selector and the encoded arguments
//         let mut data = selector;
//         data.extend(encoded_args);

//         // Generate a message
//         let call_result = session.sandbox().call_contract(
//             contract_address.clone(),
//             0,
//             data.clone(),
//             fuzzer.generate_caller(),
//             Weight::max_value() / 10,
//             None,
//             Determinism::Enforced,
//         );
//         //println!("{:?}", call_result);

//         assert!(call_result.result.is_ok());
//         let revert_flag = !call_result.result.unwrap().flags.is_empty();
//         assert!(!revert_flag);

//         // Send the same message again
//         let call_result = session.sandbox().call_contract(
//             contract_address.clone(),
//             0,
//             data,
//             fuzzer.generate_caller(),
//             Weight::max_value() / 10,
//             None,
//             Determinism::Enforced,
//         );
//         assert!(call_result.result.is_ok());
//         let revert_flag = !call_result.result.unwrap().flags.is_empty();
//         assert!(!revert_flag);

//         let get_counter_selector = fuzzer
//             .contract
//             .transcoder
//             .metadata()
//             .spec()
//             .messages()
//             .iter()
//             .find(|m| m.label() == "get_counter")
//             .unwrap()
//             .selector()
//             .to_bytes()
//             .to_vec();

//         let result = session
//             .sandbox()
//             .call_contract(
//                 contract_address.clone(),
//                 0,
//                 get_counter_selector,
//                 fuzzer.generate_caller(),
//                 Weight::max_value() / 10,
//                 None,
//                 Determinism::Enforced,
//             )
//             .result
//             .unwrap()
//             .data;

//         let counter = u128::decode(&mut &result[1..]).unwrap();
//         println!("Counter value: {}", counter);

//         // Call buggy
//         let buggy_selector = fuzzer
//             .contract
//             .transcoder
//             .metadata()
//             .spec()
//             .messages()
//             .iter()
//             .find(|m| m.label() == "buggy")
//             .unwrap()
//             .selector();

//         let buggy_selector = buggy_selector.to_bytes().to_vec();
//         let call_result = session.sandbox().call_contract(
//             contract_address.clone(),
//             0,
//             buggy_selector,
//             fuzzer.generate_caller(),
//             Weight::max_value() / 10,
//             None,
//             Determinism::Enforced,
//         );
//         assert!(call_result.result.is_ok());
//         let revert_flag = !call_result.result.unwrap().flags.is_empty();
//         assert!(!revert_flag);

//         // Check the property
//         let res = fuzzer.check_properties(&mut session, &contract_address);
//         assert!(res.is_err());
//         let error_message = res.err().unwrap().to_string();
//         println!("{:?}", error_message);
//         assert!(error_message.contains("Property check failed"));
//     }
// }
