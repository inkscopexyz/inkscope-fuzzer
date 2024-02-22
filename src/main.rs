use drink::{
    frame_support::{pallet_prelude::Encode, sp_runtime::traits::UniqueSaturatedInto}, frame_system::offchain::{Account, SendSignedTransaction}, runtime::{AccountIdFor, HashFor, MinimalRuntime}, session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT}, BalanceOf, ContractBundle
};

use hex;
use fastrand::Rng;
use log::{debug, error, info};
use std::{hash::{DefaultHasher, Hash as StdHash, Hasher}};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use scale_info::TypeDef;

//This defines all the configurable types based on the current runtime: MinimalRuntime
type Balance = BalanceOf<MinimalRuntime>;
type AccountId = AccountIdFor<MinimalRuntime>;
type Hash = HashFor<MinimalRuntime>;

type SessionBackup = Vec<u8>;
struct RuntimeFuzzer{
    rng: Rng,
    contract_path: PathBuf,
    contract: ContractBundle,
    cache: HashMap<TraceHash, SessionBackup>,
    //Settings

    potential_callers: Vec<AccountId>,
    ignore_pure_messages: bool,
}

impl RuntimeFuzzer {
    fn new(contract_path: PathBuf) -> Self {
        let contract = ContractBundle::load(&contract_path).expect("Failed to load contract");
        Self {
            rng: Rng::new(),
            contract_path,
            contract,
            cache: HashMap::new(),
            ignore_pure_messages: true,
            potential_callers: (0xA0..0xAF).map(|i| AccountId::from([i; 32])).collect(),
        }
    }

    fn fuzz_method(&mut self, ) {
        let transcoder
    };
    
    // Generates a fuzzed constructor to be prepended in the trace
    fn fuzz_constructor(&mut self) {
        let transcoder = &self.contract.transcoder;
        let metadata = transcoder.metadata();
        let constructors = metadata.spec().constructors();

        let selected_constructor =
            self.rng.choice(constructors).expect("No constructors");
        let selectec_args_spec = selected_constructor.args();

        let selector = selected_constructor.selector();
        let mut encoded = selector.to_bytes().to_vec();
        println!("Selected constructor label: {} selector: {}", selected_constructor.label(), hex::encode(selected_constructor.selector().to_bytes()));
        println!("Selected constructor args: {} ", selectec_args_spec.len());
        for arg in selectec_args_spec {
            println!("Arg: {:?} {:?}", arg.label(), arg.ty());
            let r = metadata.registry();
            let ident = arg.ty().ty().id;
            match r.resolve(ident) {
                Some(ty) => {
                    match &ty.type_def {
                        TypeDef::Composite(composite) => {
                            println!("Composite type: {:?}", composite);
                        }
                        TypeDef::Array(array) => {
                            println!("Array type: {:?}", array);
                        }
                        TypeDef::Tuple(tuple) => {
                            println!("Tuple type: {:?}", tuple);
                        }
                        TypeDef::Sequence(sequence) => {
                            println!("Sequence type: {:?}", sequence);
                        }
                        TypeDef::Variant(variant) => {
                            println!("Variant type: {:?}", variant);
                        }
                        TypeDef::Primitive(primitive) => {
                            println!("Primitive type: {:?}", primitive);
                        },
                        TypeDef::Compact(compact) => {
                            println!("Compact type: {:?}", compact);
                        },
                        TypeDef::BitSequence(bit_sequence) => {
                            println!("BitSequence type: {:?}", bit_sequence);
                        },
                    };
                }
                None => {
                    println!("Could not resolve type");
                }
                
            };

        };
    }

    //This should generate a random account id from the set of potential callers
    fn fuzz_caller(&mut self) -> AccountId {
        match self.rng.choice(&self.potential_callers){
            Some(account) => account.clone(),
            None => AccountId::from([0; 32])
        }
    }

    // Generates a fuzzed message to be added in the trace
    fn fuzz_message(&mut self) -> FuzzerCall{
        let transcoder = &self.contract.transcoder;
        let metadata = transcoder.metadata();

        // Keep only the messages that mutate the state unless ignore_pure_messages is false
        let mut messages =  metadata.spec().messages().iter().filter(|m| 
            m.mutates() ||  !self.ignore_pure_messages 
        );

        let count = messages.clone().count();
        let selected_message_idx = self.rng.usize(0..count);
        let selected_message = messages.nth(selected_message_idx).unwrap();


        println!("Selected message: {:?}", selected_message);
        FuzzerCall::Message(FuzzerMessage {
            caller: self.fuzz_caller(),
            callee: AccountId::from([0; 32]),
            endowment: Default::default(),
            input: Default::default(),
        })
    }
}

#[derive(StdHash)]
enum FuzzerCall{
    Deploy(FuzzerDeploy),
    Message(FuzzerMessage),
}

#[derive(StdHash)]
struct FuzzerDeploy {
    caller: AccountId,
    endowment: Balance,
    bytecode: Vec<u8>,
    input: Vec<u8>,
    salt: Vec<u8>,
}

#[derive(StdHash)]
struct FuzzerMessage {
    caller: AccountId,
    callee: AccountId,
    endowment: Balance,
    input: Vec<u8>,
}

// impl Hash for FuzzerCall {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.call_type.hash(state);
//         self.call_name.hash(state);
//         for arg in &self.call_args {
//             arg.hash(state);
//         }
//     }
// }

type FuzzerTrace = Vec<FuzzerCall>;
type TraceHash = u64;
fn hash_trace(trace: &FuzzerTrace) -> u64 {
    let mut hasher = DefaultHasher::new();
    trace.hash(&mut hasher);
    hasher.finish()
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fuzzer = RuntimeFuzzer::new(PathBuf::from("./flipper/target/ink/flipper.contract"));
    fuzzer.fuzz_constructor();
    fuzzer.fuzz_message();
    return Ok(());

    // TODO! use a command line argument parsing lib. Check what ink/drink ppl uses
    let contract_path = Path::new("./flipper/target/ink/flipper.contract");
    let contract: ContractBundle =
        ContractBundle::load(contract_path).expect("Failed to load contract");

    let transcoder = contract.transcoder;
    let metadata = transcoder.metadata();
    println!("type of metadata: {:?}", metadata);
    let constructors = metadata.spec().constructors();

    for constructor in constructors {
        println!("Constructor: {:?}", constructor);
    }

    let messages = metadata.spec().messages();
    for message in messages {
        println!("Message: {:?}", message);
    }

    let mut rng = Rng::new();

    let mut session = Session::<MinimalRuntime>::new()?;
    let account = session.get_actor();
    

    let selected_constructor_spec = rng.choice(constructors).expect("No constructors");
    println!("Selected constructor: {:?}", selected_constructor_spec);

    return Ok(());

    //session.deploy_bundle(       , "new", &["true"], NO_SALT, NO_ENDOWMENT)?;

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

    let account = session.deploy_bundle(contract, "new", &["true"], NO_SALT, NO_ENDOWMENT)?;

    let init_value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
    println!("Initial value: {}", init_value);

    session.call("flip", NO_ARGS, NO_ENDOWMENT)??;

    let value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
    println!("Value after flip: {}", value);

    Ok(())
}
