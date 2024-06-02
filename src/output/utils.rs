use crate::{
    contract_bundle::ContractBundle,
    engine::{
        DeployOrMessage,
        FailReason,
        FailedTrace,
    },
};
use contract_transcode::Value;

pub fn print_failed_trace(contract: &ContractBundle, failed_trace: &FailedTrace) {
    // Messages
    for (idx, deploy_or_message) in failed_trace.trace.messages.iter().enumerate() {
        print!("  Message{}: ", idx);
        print_failed_deploy_or_message(contract, deploy_or_message);
    }

    match &failed_trace.reason {
        FailReason::Trapped => {
            println!("Last message in trace has Trapped")
        }
        FailReason::Property(failed_property) => {
            // Failed properties

            match contract.decode_message(&failed_property.input) {
                Err(_e) => {
                    println!("Raw message: {:?}", &failed_property.input);
                }
                Result::Ok(x) => {
                    print!("  Property: ",);
                    print_value(&x);
                    println!();
                }
            }
        }
    };
}

pub fn print_failed_deploy_or_message(
    contract: &ContractBundle,
    deploy_or_message: &DeployOrMessage,
) {
    match deploy_or_message {
        DeployOrMessage::Deploy(deploy) => {
            let decode_result = contract.decode_deploy(&deploy.data);
            match decode_result {
                Err(_e) => {
                    println!("Raw deploy: {:?}", &deploy.data);
                }
                Result::Ok(x) => {
                    print_value(&x);
                    println!();
                }
            }
        }
        DeployOrMessage::Message(message) => {
            let decode_result = contract.decode_message(&message.input);
            match decode_result {
                Err(_e) => {
                    println!("Raw message: {:?}", &message.input);
                }
                Result::Ok(x) => {
                    print_value(&x);
                    println!();
                }
            }
        }
    }
}

pub fn print_value(value: &Value) {
    match value {
        Value::Map(map) => {
            print!("{}(", map.ident().unwrap());
            for (n, (_name, value)) in map.iter().enumerate() {
                if n != 0 {
                    print!(", ");
                }
                print_value(value);
            }
            print!(")");
        }
        _ => {
            print!("{:?}", value);
        }
    }
}
