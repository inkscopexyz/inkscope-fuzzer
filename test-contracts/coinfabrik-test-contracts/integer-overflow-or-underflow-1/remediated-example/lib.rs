#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod integer_overflow_underflow {

    #[ink(storage)]
    pub struct IntegerOverflowUnderflow {
        /// Stored value.
        value: u8,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// An overflow was produced while adding
        OverflowError,
        /// An underflow was produced while substracting
        UnderflowError,
    }

    impl IntegerOverflowUnderflow {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new(value: u8) -> Self {
            Self { value }
        }

        /// Increments the stored value by the given amount.
        #[ink(message)]
        pub fn add(&mut self, value: u8) -> Result<(), Error> {
            match self.value.checked_add(value) {
                Some(v) => self.value = v,
                None => return Err(Error::OverflowError),
            };
            Ok(())
        }

        /// Decrements the stored value by the given amount.
        #[ink(message)]
        pub fn sub(&mut self, value: u8) -> Result<(), Error> {
            match self.value.checked_sub(value) {
                Some(v) => self.value = v,
                None => return Err(Error::UnderflowError),
            };
            Ok(())
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
            let _ = self.add(value);

            // Return false if it overflowed
            init_value <= self.value
        }

        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_underflow(&mut self, value: u8) -> bool {
            // Save the initial value
            let init_value = self.value;

            // Subtract the received value
            let _ = self.sub(value);

            // Return false if it underflowed
            init_value >= self.value
        }
    }
}
