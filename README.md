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

> For other installation methods, refer to the [installation](book/src/installation.md) section of our docs.

3. Write the properties that you want to test and compile your contract with the `fuzz-testing` feature enabled.
    - Refer to the [writing properties](book/src/writing-properties.md) section of our docs for more information.

4. Run the fuzzer
```bash
    inkscope-fuzzer /path/to/file.contract fuzz
```

You can also use multiple subcommands to run the fuzzer
```bash
Usage: inkscope-fuzzer <CONTRACT> fuzz [OPTIONS]

Options:
  -c, --config <CONFIG>  Custom configuration yaml file
  -t, --tui              Enable TUI
  -o, --output <OUTPUT>  Output file for fuzzing campaign result (in 'results' dir, default: 'failed_traces')
  -h, --help             Print help
```

JSON Output File

The failed traces found during the fuzzing process are dumped as a JSON file to the results folder, use the --output flag followed by the desired filename. For example:
```bash
    inkscope-fuzzer /path/to/file.contract fuzz --output my_failed_traces.json
```
The resulting JSON file for failed traces contains a sequence of raw messages that make up a trace, followed by the reason why the trace is considered a failure (either a failed property or a trapped message).

```json
[
  {
    "trace": {
      "messages": [
        {
          "Deploy": {
            "caller": "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT",
            "endowment": 0,
            "contract_bytes": [0, 97, 115, 109, 1, 0, 0, ...],
            "data": [237, 75, 157, 27],
            "salt": [],
            "code_hash": "0x37d918287d934ceb6e8774fa3ed944f69f987fdde28cec7da81551b79030c122",
            "address": "5G9i2G3x1ziE7bfQU5LEioEayKfvwthK28JzQrLpcLjGkPT3"
          }
        },
        {
          "Message": {
            "caller": "5C7LYpP2ZH3tpKbvVvwiVe54AapxErdPBbvkYhe6y9ZBkqWt",
            "callee": "5G9i2G3x1ziE7bfQU5LEioEayKfvwthK28JzQrLpcLjGkPT3",
            "endowment": 0,
            "input": [
              124, 51, 208, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
          }
        },
        {
          "Message": {
            "caller": "5C7LYpP2ZH3tpKbvVvwiVe54AapxErdPBbvkYhe6y9ZBkqWt",
            "callee": "5G9i2G3x1ziE7bfQU5LEioEayKfvwthK28JzQrLpcLjGkPT3",
            "endowment": 0,
            "input": [
              124, 51, 208, 49, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
          }
        },
        {
          "Message": {
            "caller": "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT",
            "callee": "5G9i2G3x1ziE7bfQU5LEioEayKfvwthK28JzQrLpcLjGkPT3",
            "endowment": 0,
            "input": [
              124, 51, 208, 49, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
          }
        },
        ...
      ]
    },
    "reason": {
      "Property": {
        "caller": "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT",
        "callee": "5G9i2G3x1ziE7bfQU5LEioEayKfvwthK28JzQrLpcLjGkPT3",
        "endowment": 0,
        "input": [239, 157, 158, 137]
      }
    }
  }
]
``` 

If you want to decode the raw messages, you can use the execute command and provide the JSON file as input. The fuzzer will replay the failed traces (WIP) and print the decoded messages.

```bash
    inkscope-fuzzer /path/to/file.contract execute ./results/my_failed_traces.json
```

```bash
Executing contract: "./test-contracts/ityfuzz/target/ink/ityfuzz.contract"
Using input: "./results/my_failed_traces.json"
Executing failed trace 0
  Message0: default()
  Message1: incr(UInt(0))
  Message2: incr(UInt(1))
  Message3: incr(UInt(1))
  Message4: incr(UInt(2))
  Message5: incr(UInt(2))
  Message6: incr(UInt(2))
  Message7: incr(UInt(1))
  Message8: incr(UInt(2))
  Message9: decr(UInt(340282366920938463463374607431768211455))
  Message10: incr(UInt(2))
  Message11: incr(UInt(1))
  Message12: incr(UInt(1))
  Message13: incr(UInt(2))
  Message14: incr(UInt(2))
  Message15: incr(UInt(0))
  Message16: incr(UInt(2))
  Message17: incr(UInt(0))
  Message18: buggy()
  Property: inkscope_bug()
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

> **Note:** To compile and run the test contracts manually in your local environment, ensure that you're using cargo-contract version 4.1.1.

### 🎨 Text User Interface

You can start a TUI by passing --tui in the command line (or changing the `use_tui` variable in the config.yaml) 
```bash
    inkscope-fuzzer  ./test-contracts/ityfuzz/target/ink/ityfuzz.contract fuzz --tui
```
![image](https://github.com/inkscopexyz/inkscope-fuzzer/assets/1017522/96a51639-3150-4dcb-a308-a5fe5d320870)
