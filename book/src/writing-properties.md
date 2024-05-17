# Writing properties

Inkscope fuzzer supports 3 approaches for writing properties to detect bugs in ink! smart contracts:

1. **Dedicated Property Messages (Recommended):**  
The recommended approach is to write a new message that performs a specific action and always returns a boolean value. If the function returns false, it indicates that the property has been violated, signaling the presence of a bug. These properties can also execute other functions that modify the contract state, but the state changes are not saved as they are performed in a dry run during the fuzzing process.  
For convention, these messages name should start with `inkscope_` to differentiate them from regular contract messages.

    *Example:*
    ```rust
    #![cfg_attr(not(feature = "std"), no_std, no_main)]

    #[ink::contract]
    mod example_contract {
      
        #[ink(storage)]
        pub struct ExampleContract {
           // State variables
        }

        impl ExampleContract {
            // Regular contract constructors and messages
            ...
        }

        // Dedicated property messages for fuzz testing
        #[cfg(feature = "fuzz-testing")]
        #[ink(impl)]
        impl ExampleContract {
            #[cfg(feature = "fuzz-testing")]
            #[ink(message)]
            pub fn inkscope_bug(&self) -> bool {
                // Property logic that returns true or false
            }
        }

    }
    ```

2. **Assertions in Existing Calls:**  
An alternative method is to incorporate assert statements within the existing contract calls. If any of these assertions fail, it means a bug has been discovered in the contract.

    *Example:*
    ```rust
    #![cfg_attr(not(feature = "std"), no_std, no_main)]

    #[ink::contract]
    mod example_contract {
      
        #[ink(storage)]
        pub struct ExampleContract {
           value: u128,
        }

        impl ExampleContract {
            // Regular contract constructors and messages
            #[ink(constructor)]
            pub fn new(init_value: u128) -> Self {
                Self { value: init_value }
            }

            #[ink(message)]
            pub fn incr_value(&self, value: u128) {
                self.value += value;

                // Write assertions to check property
                assert!(self.value <= 10, "Value must be less than or equal to 10");
            }
            
            ...

        }

    }
    ```

3. **Panic-based Properties:**  
The third approach involves making the contract panic when a specific condition is met. This signals to the fuzzer that a property has been violated and a bug has been found.

    *Example:*
    ```rust
    #![cfg_attr(not(feature = "std"), no_std, no_main)]

    #[ink::contract]
    mod example_contract {
      
        #[ink(storage)]
        pub struct ExampleContract {
           value: u128,
        }

        impl ExampleContract {
            // Regular contract constructors and messages
            #[ink(constructor)]
            pub fn new(init_value: u128) -> Self {
                Self { value: init_value }
            }

            #[ink(message)]
            pub fn incr_value(&self, value: u128) {
                self.value += value;

                // Check property and panic if violated
                if self.value > 10 {
                    panic!("Value must be less than or equal to 10");
                }
            }
            
            ...

        }

    }
    ```

The choice of property-writing approach depends on the specific requirements and complexity of the smart contract being tested. The dedicated property messages method is generally recommended as it provides a clear separation of concerns and makes the testing process more organized and maintainable.