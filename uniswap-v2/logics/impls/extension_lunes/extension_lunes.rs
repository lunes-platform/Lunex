
use openbrush::{
    contracts::psp22::PSP22Error,
    traits::{
        AccountId,
        Balance,
    },
};

pub use crate::{
    impls::extension_lunes::*,
    traits::extension_lunes::*,
};

use ink::prelude::vec::Vec;

#[openbrush::trait_definition]
pub trait Psp22ExtensionLunes{
        #[ink(message)]
        fn total_supply(&self) -> Balance;

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance;

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), PSP22Error>;

        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(),PSP22Error>;

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(),PSP22Error>;

        #[ink(message)]
        fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> Result<(),PSP22Error>;

        #[ink(message)]
        fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> Result<(),PSP22Error>;

        // Metadata interfaces
        #[ink(message)]
        fn token_name(&self) -> Result<Vec<u8>,PSP22Error>;

        #[ink(message)]
        fn token_symbol(&self) -> Result<Vec<u8>,PSP22Error>;

        #[ink(message)]
        fn token_decimals(&self) -> Result<u8,PSP22Error>;

        #[ink(message)]
        fn mint(&mut self, to: AccountId, value: Balance) -> Result<(),PSP22Error>;

        #[ink(message)]
        fn burn(&mut self, from: AccountId, value: Balance) -> Result<(),PSP22Error>;
}
