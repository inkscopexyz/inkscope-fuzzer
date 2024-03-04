#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod primitive_generator_tester {

    #[ink(storage)]
    pub struct PrimitiveGeneratorTester {}

    impl PrimitiveGeneratorTester {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        /// A message to test the bool type generator
        #[ink(message)]
        pub fn bool_message(&mut self, bool_value: bool) {}

        // /// A message to test the char type generator
        // #[ink(message)]
        // pub fn char_message(&mut self, char_value: char) {} // the trait `WrapperTypeDecode` is not implemented for `char`

        // /// A message to test the str type generator
        // #[ink(message)]
        // pub fn str_message(&mut self, str_value: &str) {} // the trait `WrapperTypeDecode` is not implemented for `&str`

        // /// A message to test the u8 type generator
        // #[ink(message)]
        // pub fn u8_message(&mut self, u8_value: u8) {}

        // /// A message to test the u16 type generator
        // #[ink(message)]
        // pub fn u16_message(&mut self, u16_value: u16) {}

        // /// A message to test the u32 type generator
        // #[ink(message)]
        // pub fn u32_message(&mut self, u32_value: u32) {}

        // /// A message to test the u64 type generator
        // #[ink(message)]
        // pub fn u64_message(&mut self, u64_value: u64) {}

        // /// A message to test the u128 type generator
        // #[ink(message)]
        // pub fn u128_message(&mut self, u128_value: u128) {}

        // /// A message to test the i8 type generator
        // #[ink(message)]
        // pub fn i8_message(&mut self, i8_value: i8) {}

        // /// A message to test the i16 type generator
        // #[ink(message)]
        // pub fn i16_message(&mut self, i16_value: i16) {}

        // /// A message to test the i32 type generator
        // #[ink(message)]
        // pub fn i32_message(&mut self, i32_value: i32) {}

        // /// A message to test the i64 type generator
        // #[ink(message)]
        // pub fn i64_message(&mut self, i64_value: i64) {}

        // /// A message to test the i128 type generator
        // #[ink(message)]
        // pub fn i128_message(&mut self, i128_value: i128) {}
    }
}
