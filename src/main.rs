
use drink::{
    frame_support::pallet_prelude::Encode, runtime::MinimalRuntime, session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT}, ContractBundle};

use drink::session::contract_transcode::ContractMessageTranscoder;


use std::{collections::HashMap, path::Path};
use fastrand::Rng;
use log::{info, debug, error};
use std::hash::{Hash, DefaultHasher, Hasher};
use std::path::PathBuf;

type TraceHash = u64;
fn hash_trace(trace: &FuzzerTrace)-> u64{
    let mut hasher = DefaultHasher::new();
    trace.hash(&mut hasher);
    hasher.finish()
}

type SessionBackup = Vec<u8>;
struct RuntimeFuzzer{
    rng: Rng,
    contract_path: PathBuf,
    contract: ContractBundle,
    cache: HashMap<TraceHash, SessionBackup>,
}

#[derive(Hash)]
enum FuzzerCallType{
    Constructor,
    Message,
}

struct FuzzerCall{   
    call_type: FuzzerCallType,
    call_name: String,
    call_args: Vec<String>,
}

impl Hash for FuzzerCall{
    fn hash<H: Hasher>(&self, state: &mut H){
        self.call_type.hash(state);
        self.call_name.hash(state);
        for arg in &self.call_args{
            arg.hash(state);
        }
    }
}

type FuzzerTrace = Vec<FuzzerCall>;


// function that expects a transcoderand returns a vec os message specs
fn get_message_specs(transcoder: &ContractMessageTranscoder) -> Vec<String>{
    let metadata = transcoder.metadata();
    let messages = metadata.spec().messages();
    let mut message_specs = Vec::new();
    for message in messages{
        message_specs.push(message.label().to_string());
    }
    message_specs
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO! use a command line argument parsing lib. Check what ink/drink ppl uses
    let contract_path = Path::new("./flipper/target/ink/flipper.contract");
    let contract: ContractBundle = ContractBundle::load(contract_path).expect("Failed to load contract");
    
    let transcoder = contract.transcoder;
    let metadata = transcoder.metadata();
    println!("type of metadata: {:?}", metadata);
    let constructors = metadata.spec().constructors();

    
    for constructor in constructors{
        println!("Constructor: {:?}", constructor);
    }

    let messages = metadata.spec().messages();
    for message in messages{
        println!("Message: {:?}", message);
    }

    let mut rng = Rng::new();

    let mut session = Session::<MinimalRuntime>::new()?;
  

    let selected_constructor_spec = rng.choice(constructors).expect("No constructors");
    println!("Selected constructor: {:?}", selected_constructor_spec);

    return Ok(());

    session.deploy_bundle(contract, "new", &["true"], NO_SALT, NO_ENDOWMENT)?;


    //List types and methods
   // let methods_description = contract.contract.methods;


    // Run deploy(fuzzed_args)
    // N = FuzzedLen()
    // for i in 0..N:
    //     Functype = rng.choice(methods_description))
    //     for arg in functypes.args:
    //          match(arg.type){
    //              int => rng.int()
    //              bool => rng.bool()
    //              string => rng.string()
    //              address => rng.address()
    //              bytes => rng.bytes()
    //          }
    //     TAKE SNAPSHOT()

    //     FOR i in 0..M:


    session.deploy_bundle(contract, "new", &["true"], NO_SALT, NO_ENDOWMENT)?;
    
   

    let init_value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
    println!("Initial value: {}", init_value);

    session.call("flip", NO_ARGS, NO_ENDOWMENT)??;

    let value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
    println!("Value after flip: {}", value);

    Ok(())
}
