use ink::prelude::vec::Vec;


#[openbrush::wrapper]
pub type Psp22ExtensionRef = dyn Psp22ExtensionLunes;
use openbrush::{
    contracts::psp22::PSP22Error,
    traits::{AccountId as DefaultAccountId, Balance as DefaultBalance},
};


#[openbrush::trait_definition]
pub trait Psp22ExtensionLunes {
    #[ink(message)]
    fn total_supply(&self) -> DefaultBalance;

    #[ink(message)]
    fn balance_of(&self, owner: DefaultAccountId) -> DefaultBalance;

    #[ink(message)]
    fn allowance(&self, owner: DefaultAccountId, spender: DefaultAccountId) -> DefaultBalance;

    #[ink(message)]
    fn transfer(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;

    #[ink(message)]
    fn transfer_from(&mut self, from: DefaultAccountId, to: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;

    #[ink(message)]
    fn approve(&mut self, spender: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;

    #[ink(message)]
    fn increase_allowance(&mut self, spender: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;

    #[ink(message)]
    fn decrease_allowance(&mut self, spender: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;

    // Metadata interfaces
    #[ink(message)]
    fn token_name(&self) -> Result<Vec<u8>,PSP22Error>;

    #[ink(message)]
    fn token_symbol(&self) -> Result<Vec<u8>,PSP22Error>;

    #[ink(message)]
    fn token_decimals(&self) -> Result<u8,PSP22Error>;

    #[ink(message)]
    fn mint(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;

    #[ink(message)]
    fn burn(&mut self, from: DefaultAccountId, value: DefaultBalance) -> Result<(),PSP22Error>;
}
