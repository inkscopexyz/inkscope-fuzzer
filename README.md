# Inkscope Fuzzer

> Note: The fuzzer is a work in progress and is not yet functional.

### To run example:

Compile smart contract with fuzzing properties:

> If the smart contract is not compiled with the fuzzing feature, or the contract does not have any fuzzing properties, the fuzzer will not run.

```bash
    cd test-contracts/flipper && cargo contract build --features fuzz-testing && cd ../..
```

Then run the fuzzer: (wip)

```bash
    cargo run
```
