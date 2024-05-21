# Inkscope Fuzzer

## Overview

Inkscope fuzzer is a property-based fuzzing tool designed to find bugs and vulnerabilities in Ink! smart contracts during the development phase. It utilizes the official ink-sandbox runtime emulation engine to execute and test Polkadot smart contracts against user-defined properties.

These properties are writen in ink! itself and the fuzzer starts from a .contract file produced from a ink compilation. The fuzzer generates random inputs and checks if the provided properties hold true for the smart contract under test.

If the fuzzer discovers a property violation, it prints the complete execution trace, including the contract deployment process, all the messages called, and the violated properties. This detailed output assists developers in identifying and fixing issues within their smart contracts.

By incorporating property-based testing through Inkscope fuzzer, developers can enhance the reliability and security of their Ink! smart contracts before deployment on the Polkadot network.

>  The ink contract you want to fuzz-test must use ^5.0.0 version of the ink! crate

### 🚀 How to run the fuzzer

1. Clone this repository and enter the project folder
```bash
    git clone https://github.com/inkscopexyz/inkscope-fuzzer.git && cd inkscope-fuzzer
```

2. Install the fuzzer using cargo
```bash
    cargo install --path .
```

> The `inkscope-fuzzer` executable will be ready to use in your system.

3. Write the properties that you want to test and compile your contract with the `fuzz-testing` feature enabled.
    - Refer to the [writing properties](book/src/writing-properties.md) section of our docs for more information.

4. Run the fuzzer
```bash
    inkscope-fuzzer /path/to/file.contract
```

> For other installation methods, refer to the [installation](book/src/installation.md) section of our docs.
5. Optional. Generate a json output file
In order to dump any failed trace as a json file used --output FILENAME like so.
```bash
    inkscope-fuzzer --output falied_traces.json /path/to/file.contract
```
The resultant json file for failed traces should be relativelly straigh forward to process. It is a secuence of messages making a trace followed by the reason why the trace is considered a fail (property that failed or message that trapped).
```
[
  {
    "trace": {
      "messages": [
        {
          "Deploy": {
            "caller": "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT",
            "endowment": 0,
            "contract_bytes": [...],
          }
        },
        {
          "Message": {
            "caller": "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT",
            "callee": "5FKacgmGEFtBhEMvHshr1rZtyg7DC1zwTSmSgAC8dX1xHPfb",
            "endowment": 0,
            "input": [ 148, 35, 33,115 ]
          }
        },
        ...
      ]
      "reason": {
         "Property": {
            "caller": "5C7LYpP2ZH3tpKbvVvwiVe54AapxErdPBbvkYhe6y9ZBkqWt",
            "callee": "5FKacgmGEFtBhEMvHshr1rZtyg7DC1zwTSmSgAC8dX1xHPfb",
            "endowment": 0,
            "input": [239, 157, 158, 137 ]
        }
        ...

``` 



### Initial Example

Check out the [Ityfuzz](book/src/ityfuzz.md) contract challenge to understand how the fuzzer works.

### ⚙️ Testing

- ⚠️ Requirements:
  - Docker

In order to test the fuzzer, you need to follow the steps below:

1. Build the testing docker image
```bash
    docker build -t inkscope-fuzzer-testing -f ./.docker/testing/Dockerfile .
```
2. Run the tests
```bash
    docker run inkscope-fuzzer-testing
```

### 🎨 Text User Interface

You can start a TUI by passing --tui in the command line (or changing the `use_tui` variable in the config.yaml) 
```bash
    inkscope-fuzzer  ./test-contracts/ityfuzz/target/ink/ityfuzz.contract --tui
```
![image](https://github.com/inkscopexyz/inkscope-fuzzer/assets/1017522/96a51639-3150-4dcb-a308-a5fe5d320870)
