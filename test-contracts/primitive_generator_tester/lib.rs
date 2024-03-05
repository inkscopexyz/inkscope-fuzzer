#![cfg_attr(not(feature = "std"), no_std, no_main)]


#[ink::contract]
mod primitive_generator_tester {
    const BOOL_CONSTANT: bool = true;

    #[ink(storage)]
    pub struct PrimitiveGeneratorTester {
        flag: bool
    }

    impl PrimitiveGeneratorTester {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                flag: false
            }
        }

        /// A message to test the bool type generator
        #[ink(message)]
        pub fn bool_message(&mut self, bool_value: bool) {
            if bool_value == BOOL_CONSTANT{
                self.flag = true;
            }
        }
    }
    #[cfg(feature = "fuzz-testing")]
    #[ink(impl)]
    impl PrimitiveGeneratorTester {
        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_property(&self) -> bool {
            !self.flag
        }
    }
}
