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

        // Multiply the stored value by the given amount.
        #[ink(message)]
        pub fn mul(&mut self, value: u8) -> Result<(), Error> {
            match self.value.checked_mul(value) {
                Some(v) => self.value = v,
                None => return Err(Error::OverflowError),
            };
            Ok(())
        }

        // Raise the stored value to the power of the given amount.
        #[ink(message)]
        pub fn pow(&mut self, value: u8) -> Result<(), Error> {
            match self.value.checked_pow(value.into()) {
                Some(v) => self.value = v,
                None => return Err(Error::OverflowError),
            };
            Ok(())
        }

        // Negate the stored value.
        #[ink(message)]
        pub fn neg(&mut self) -> Result<(), Error> {
            match self.value.checked_neg() {
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
        pub fn inkscope_mul_overflows(&mut self, value: u8) -> bool {
            // If the value is 0, it will never overflow
            if value == 0 {
                return true;
            }

            // Save the initial value
            let init_value = self.value;

            // Add the received value
            self.mul(value);

            // Return false if it overflowed
            init_value <= self.value
        }

        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_pow_overflows(&mut self, value: u8) -> bool {
            // If the value is 0, it will never overflow
            if value == 0 {
                return true;
            }

            // Save the initial value
            let init_value = self.value;

            // Subtract the received value
            self.pow(value);

            // Return false if it overflowed
            init_value <= self.value
        }

        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_neg_overflows(&mut self) -> bool {
            // Save the initial value
            let init_value = self.value;

            // Subtract the received value
            self.neg();

            // Return false if it overflowed
            init_value >= self.value
        }
    }
}
