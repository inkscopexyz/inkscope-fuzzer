#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ityfuzz {

    const BUG_VALUE: u128 = 2;

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
            if (value > self.counter) {
                return Err(());
            }
            self.counter = self.counter.checked_add(1).ok_or(())?;
            Ok(())
        }

        #[ink(message)]
        pub fn decr(&mut self, value: u128) -> Result<(), ()> {
            if (value < self.counter) {
                return Err(());
            }
            self.counter = self.counter.checked_sub(1).ok_or(())?;
            Ok(())
        }

        #[ink(message)]
        pub fn buggy(&mut self) {
            if (self.counter == BUG_VALUE) {
                self.bug_flag = false;
            }
        }

        #[ink(message)]
        pub fn get_counter(&self) -> u128 {
            self.counter
        }
    }

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
