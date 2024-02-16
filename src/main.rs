use drink::{
    runtime::MinimalRuntime,
    session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    ContractBundle,
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::<MinimalRuntime>::new()?;
    let contract_path = Path::new("./flipper/target/ink/flipper.contract");
    let contract = ContractBundle::load(contract_path).expect("Failed to load contract");
    let init_value: bool = session
        .deploy_bundle_and(contract, "new", &["true"], NO_SALT, NO_ENDOWMENT)?
        .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
        .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
        .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
        .call_and("get", NO_ARGS, NO_ENDOWMENT)?
        .record()
        .last_call_return_decoded()?
        .expect("Call was successful");

    Ok(())
}
