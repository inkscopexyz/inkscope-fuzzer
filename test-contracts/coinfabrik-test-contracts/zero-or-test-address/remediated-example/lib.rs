#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod zerocheck {

    #[ink(storage)]
    pub struct Zerocheck {
        admin: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Caller is not not authorized.
        NotAuthorized,
        /// Address is invalid.
        InvalidAddress,
    }

    impl Zerocheck {
        #[ink(constructor)]
        pub fn new() -> Self {
            let admin = Self::env().caller();
            Self { admin }
        }

        /// Changes the admin and returns the new admin. Reverts if 0x0... is the new admin.
        #[ink(message)]
        pub fn modify_admin(&mut self, admin: AccountId) -> Result<AccountId, Error> {
            if self.admin != self.env().caller() {
                return Err(Error::NotAuthorized);
            }

            if admin == AccountId::from([0x0; 32]) {
                return Err(Error::InvalidAddress);
            }

            self.admin = admin;
            Ok(self.admin)
        }
    }

    impl Default for Zerocheck {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(feature = "fuzz-testing")]
    #[ink(impl)]
    impl Zerocheck {
        #[cfg(feature = "fuzz-testing")]
        #[ink(message)]
        pub fn inkscope_zero_check(&self) -> bool {
            !(self.admin == AccountId::from([0x0; 32]))
        }
    }    
}