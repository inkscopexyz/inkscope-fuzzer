#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod assert_violation {

    #[ink(storage)]
    pub struct AssertViolation {
        value: u128,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        GreaterThan10,
    }

    impl AssertViolation {
        #[ink(constructor)]
        pub fn new(init_value: u128) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn revert_if_greater_than_10(&self, value: u128) -> Result<bool, Error> {
            if value <= 10 {
                Ok(true)
            } else {
                Err(Error::GreaterThan10)
            }
        }
    }
}