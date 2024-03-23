#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod SimpleFlag {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct SimpleFlag {
        /// Stores a single `bool` value on the storage.
        flag: bool,
    }

    impl SimpleFlag {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { flag: false }
        }


        #[ink(message)]
        pub fn flip_bool(&mut self, arg_bool: bool) {
            self.flag = true;
        }
        
    }

    #[cfg(feature = "fuzz-testing")]
    #[ink(impl)]
    impl SimpleFlag {
        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_property_1(&self) -> bool {
            true
        }
        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_property_2(&self) -> bool {
            false
        }
    }

}
