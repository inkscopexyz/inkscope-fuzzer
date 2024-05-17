# Ityfuzz

Let's start with a simple example to understand how the fuzzer works.

- [Ityfuzz contract challenge](https://dl.acm.org/doi/pdf/10.1145/3597926.3598059)
> Check section 3 Figure 2 for the pseudo-code of the contract.

Here, we reproduce the contract in ink!:

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

In this contract, the `incr` and `decr` functions increment and decrement the `counter` variable based on the condition compared to the provided value, respectively. The `buggy` function sets the `bug_flag` variable to false if the `counter` variable is equal to `BUG_VALUE`.

## Fuzzing the contract

To test the contract we will write a property as an ink! message that checks the value of the `bug_flag` variable. If the message returns 'false', it means that property was violated, and the fuzzer will print the execution trace and the violated property.

Note that this message is wrapped in a `#[cfg(feature = "fuzz-testing")]` attribute to avoid compiling it in the final contract. In order for this to work, the `fuzz-testing` feature must be enabled in the `Cargo.toml` file.

```toml
[features]
...
fuzz-testing = []
```
And this is how the property looks like in the ink! contract: 
```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ityfuzz {
    ...

    #[cfg(feature = "fuzz-testing")]
    #[ink(impl)]
    impl Ityfuzz {
        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_bug(&self) -> bool {
            self.bug_flag
        }
    }
}
```

Once the property is written, we can compile the contract:

```bash
    cd test-contracts/ityfuzz
    cargo contract build --features fuzz-testing
```

And then, execute the fuzzer against it and check the output
```bash
    ./target/release/inkscope-fuzzer ./test-contracts/ityfuzz/target/ink/ityfuzz.contract 
```

If the fuzzer finds a property violation, it will print the execution trace and the violated property as shown below:

```bash
Property check failed ❌
  Deploy: default()
  Message0:   Deploy: incr(UInt(0))
  Message1:   Deploy: buggy()
  Message2:   Deploy: incr(UInt(1))
  Message3:   Deploy: buggy()
  Message4:   Deploy: incr(UInt(1))
  Message5:   Deploy: buggy()
  Message6:   Deploy: buggy()
  Message7:   Deploy: buggy()
  Message8:   Deploy: buggy()
  Message9:   Deploy: incr(UInt(1))
  Message10:   Deploy: buggy()
  Message11:   Deploy: incr(UInt(0))
  Message12:   Deploy: incr(UInt(1))
  Message13:   Deploy: buggy()
  Message14:   Deploy: incr(UInt(1))
  Message15:   Deploy: incr(UInt(2))
  Message16:   Deploy: buggy()
  Message17:   Deploy: buggy()
  Message18:   Deploy: incr(UInt(1))
  Message19:   Deploy: incr(UInt(0))
  Message20:   Deploy: incr(UInt(2))
  Message21:   Deploy: incr(UInt(1))
  Message22:   Deploy: buggy()
  Message23:   Deploy: buggy()
  Message24:   Deploy: buggy()
  Message25:   Deploy: incr(UInt(2))
  Message26:   Deploy: incr(UInt(1))
  Message27:   Deploy: decr(UInt(340282366920938463463374607431768211455))
  Message28:   Deploy: incr(UInt(1))
  Message29:   Deploy: incr(UInt(1))
  Message30:   Deploy: buggy()
  Property: inkscope_bug()
```
