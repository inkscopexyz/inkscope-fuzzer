#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod integer_overflow_underflow {

    #[ink(storage)]
    pub struct IntegerOverflowUnderflow {
        /// Stored value.
        value: u8,
    }

    impl IntegerOverflowUnderflow {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new(value: u8) -> Self {
            Self { value }
        }

        /// Increments the stored value by the given amount.
        #[ink(message)]
        pub fn add(&mut self, value: u8) {
            self.value += value;
        }

        /// Decrements the stored value by the given amount.
        #[ink(message)]
        pub fn sub(&mut self, value: u8) {
            self.value -= value;
        }

        /// Returns the stored value.
        #[ink(message)]
        pub fn get(&self) -> u8 {
            self.value
        }
    }

    #[cfg(feature = "fuzz-testing")]
    #[ink(impl)]
    impl IntegerOverflowUnderflow {
        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_overflow(&mut self, value: u8) -> bool {
            // Save the initial value
            let init_value = self.value;

            // Add the received value
            self.add(value);

            // Return false if it overflowed
            init_value <= self.value
        }

        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_underflow(&mut self, value: u8) -> bool {
            // Save the initial value
            let init_value = self.value;

            // Subtract the received value
            self.sub(value);

            // Return false if it underflowed
            init_value >= self.value
        }
    }
}
