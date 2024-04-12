# Inkscope Fuzzer

*Documentation for inkscope-fuzzer users and developers.*

Inkscope fuzzer is a property-based fuzzing tool designed to find bugs and vulnerabilities in Ink! smart contracts during the development phase. It utilizes the ink-sandbox runtime emulation engine to execute and test Polkadot smart contracts against user-defined properties.  

These properties are written in ink! and the fuzzer starts from a .contract file produced from the compilation. The fuzzer generates random inputs and checks if the provided properties hold true for the smart contract under test.

If the fuzzer discovers a property violation, it prints the complete execution trace, including the contract deployment process, all the messages called, and the violated properties. This detailed output assists developers in identifying and fixing issues within their contracts.

By incorporating property-based testing through inkscope fuzzer, developers can enhance the reliability and security of their smart contracts before deployment on a live network.

