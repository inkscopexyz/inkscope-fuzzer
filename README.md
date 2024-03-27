# Inkscope Fuzzer

## Overview

Inkscope fuzzer is a property-based fuzzing tool designed to find bugs and vulnerabilities in Ink! smart contracts during the development phase. It utilizes the Drink runtime emulation engine to execute and test Polkadot smart contracts against user-defined properties. 

These properties are writen in ink! and the fuzzer starts from a .contract file produced from a ink compilation. The fuzzer generates random inputs and checks if the provided properties hold true for the smart contract under test.

If the fuzzer discovers a property violation, it prints the complete execution trace, including the contract deployment process, all the messages called, and the violated properties. This detailed output assists developers in identifying and fixing issues within their smart contracts.

By incorporating property-based testing through Inkscope fuzzer, developers can enhance the reliability and security of their Ink! smart contracts before deployment on the Polkadot network.

## Initial Example

Let's start with a simple example to understand how the fuzzer works.


- Ityfuzz contract challenge

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ityfuzz {

    const BUG_VALUE: u128 = 15;

    #[ink(storage)]
    pub struct Ityfuzz {
        counter: u128,
        bug_flag: bool,
    }

    impl Ityfuzz {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                counter: 0,
                bug_flag: true,
            }
        }

        #[ink(message)]
        pub fn incr(&mut self, value: u128) -> Result<(), ()> {
            if value > self.counter {
                return Err(());
            }
            self.counter = self.counter.checked_add(1).ok_or(())?;
            Ok(())
        }

        #[ink(message)]
        pub fn decr(&mut self, value: u128) -> Result<(), ()> {
            if value < self.counter {
                return Err(());
            }
            self.counter = self.counter.checked_sub(1).ok_or(())?;
            Ok(())
        }

        #[ink(message)]
        pub fn buggy(&mut self) {
            if self.counter == BUG_VALUE {
                self.bug_flag = false;
            }
        }

        #[ink(message)]
        pub fn get_counter(&self) -> u128 {
            self.counter
        }
    }
}

```

Compile the test contracts
```bash
    cd test-contracts
    . ./build.sh
```

Execute the fuzzer against it and check the output
```bash
    ./target/release/inkscope-fuzzer ./test-contracts/ityfuzz/target/ink/ityfuzz.contract 
```

- ⚠️ Requirements:
  - rust = 1.76
  - cargo-contract 3.2.0
  - The ink contract you want to fuzz-test must use ^5.0.0 version of the ink! crate

### ⚙️ Test

In order to test the fuzzer, you need to follow the steps below:

1. Compile the test contracts
```bash
    cd test-contracts
    . ./build.sh
```
2. Run the tests
```bash
    cargo test
```

### 🚀 Run the fuzzer

#### A. Development

1. Clone this repository and enter the project folder
```bash
    git clone https://github.com/inkscopexyz/inkscope-fuzzer.git && cd inkscope-fuzzer
```
2. Build the project
```bash
    cargo build --release
```
3. Write the properties that you want to test
    - Refer to the [write-properties document](docs/write-properties.md) for more information.
    
4. Run the fuzzer
```bash
    ./target/release/inkscope-fuzzer "/path/to/file.contract"
```


