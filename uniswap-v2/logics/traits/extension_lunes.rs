use ink::prelude::vec::Vec;

use openbrush::traits::{AccountId as DefaultAccountId, Balance as DefaultBalance};
#[openbrush::wrapper]
pub type Psp22ExtensionRef = dyn Psp22Extension;

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Psp22Error {
    TotalSupplyFailed,
}

pub type Result<T> = core::result::Result<T, Psp22Error>;

#[openbrush::trait_definition]
pub trait Psp22Extension {
    #[ink(message)]
    fn total_supply(&self) -> DefaultBalance;

    #[ink(message)]
    fn balance_of(&self, owner: DefaultAccountId) -> DefaultBalance;

    #[ink(message)]
    fn allowance(&self, owner: DefaultAccountId, spender: DefaultAccountId) -> DefaultBalance;

    #[ink(message)]
    fn transfer(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    #[ink(message)]
    fn transfer_from(&mut self, from: DefaultAccountId, to: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    #[ink(message)]
    fn approve(&mut self, spender: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    #[ink(message)]
    fn increase_allowance(&mut self, spender: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    #[ink(message)]
    fn decrease_allowance(&mut self, spender: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // Metadata interfaces
    #[ink(message)]
    fn token_name(&self) -> Result<Vec<u8>>;

    #[ink(message)]
    fn token_symbol(&self) -> Result<Vec<u8>>;

    #[ink(message)]
    fn token_decimals(&self) -> Result<u8>;

    #[ink(message)]
    fn mint(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    #[ink(message)]
    fn burn(&mut self, from: DefaultAccountId, value: DefaultBalance) -> Result<()>;
}
