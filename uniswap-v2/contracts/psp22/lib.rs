#![cfg_attr(not(feature = "std"), no_std, no_main)]
use ink::{
    env::Environment,
    prelude::vec::Vec,
};

type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[ink::chain_extension]
pub trait Psp22Extension {
    type ErrorCode = Psp22Error;

    // PSP22 Metadata interfaces

    #[ink(extension = 0x3d26)]
    fn token_name(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(extension = 0x3420)]
    fn token_symbol(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(extension = 0x7271)]
    fn token_decimals(asset_id: u32) -> Result<u8>;

    // PSP22 interface queries

    #[ink(extension = 0x162d)]
    fn total_supply(asset_id: u32) -> DefaultBalance;

    #[ink(extension = 0x6568)]
    fn balance_of(asset_id: u32, owner: DefaultAccountId) -> DefaultBalance;

    #[ink(extension = 0x4d47)]
    fn allowance(
        asset_id: u32,
        owner: DefaultAccountId,
        spender: DefaultAccountId,
    ) -> DefaultBalance;

    // PSP22 transfer
    #[ink(extension = 0xdb20)]
    fn transfer(asset_id: u32, to: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // PSP22 transfer_from
    #[ink(extension = 0x54b3)]
    fn transfer_from(
        asset_id: u32,
        from: DefaultAccountId,
        to: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 approve
    #[ink(extension = 0xb20f)]
    fn approve(asset_id: u32, spender: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // PSP22 increase_allowance
    #[ink(extension = 0x96d6)]
    fn increase_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 decrease_allowance
    #[ink(extension = 0xfecb)]
    fn decrease_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 mint
    #[ink(extension = 0x6bba)]
    fn mint(
        asset_id: u32,
        to: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 mint
    #[ink(extension = 0x9e55)]
    fn burn(
        asset_id: u32,
        from: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;
    
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Psp22Error {
    TotalSupplyFailed,
    BalanceOfFailed,
    AllowanceFailed,
    TransferFailed,
    TransferFromFailed,
    ApproveFailed,
    IncreaseAllowanceFailed,
    DecreaseAllowanceFailed,
    TokenNameFailed,
    TokenSymbolFailed,
    TokenDecimalsFailed,
}

pub type Result<T> = core::result::Result<T, Psp22Error>;

impl From<scale::Error> for Psp22Error {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink::env::chain_extension::FromStatusCode for Psp22Error {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::TotalSupplyFailed),
            2 => Err(Self::BalanceOfFailed),
            3 => Err(Self::AllowanceFailed),
            4 => Err(Self::TransferFailed),
            5 => Err(Self::TransferFromFailed),
            6 => Err(Self::ApproveFailed),
            7 => Err(Self::IncreaseAllowanceFailed),
            8 => Err(Self::DecreaseAllowanceFailed),
            9 => Err(Self::TokenNameFailed),
            10 => Err(Self::TokenSymbolFailed),
            11 => Err(Self::TokenDecimalsFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

/// An environment using default ink environment types, with PSP-22 extension included
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = DefaultAccountId;
    type Balance = DefaultBalance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;

    type ChainExtension = crate::Psp22Extension;
}

pub mod psp22 {
    use ink::prelude::vec::Vec;
    use crate::{DefaultAccountId, DefaultBalance, Result};

    #[ink::trait_definition]
    pub trait PSP22 {
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
}


#[ink::contract(env = crate::CustomEnvironment)]
mod token {
    use super::{
        psp22::PSP22,
        Result,
        Vec,
        DefaultAccountId,
        DefaultBalance,
    };
    #[ink(storage)]
    #[derive(Default)]
    pub struct MyPSP22 {
        asset_id: u32,
    }

    impl MyPSP22 {
        #[ink(constructor)]
        pub fn new(asset_id: u32) -> Self {
            Self { asset_id }
        }

        #[ink(message)]
        pub fn asset_id(&self) -> u32 {
            self.asset_id
        }
    }

    impl PSP22 for MyPSP22 {
        #[ink(message)]
        fn total_supply(&self) -> DefaultBalance {           
            self.env().extension().total_supply(self.asset_id).unwrap_or(0)
        }

        #[ink(message)]
        fn balance_of(&self, owner: DefaultAccountId) -> DefaultBalance {
            self.env().extension().balance_of(self.asset_id, owner).unwrap_or(0)
        }

        #[ink(message)]
        fn allowance(
            &self,
            owner: DefaultAccountId,
            spender: DefaultAccountId,
        ) -> DefaultBalance {
            self.env().extension().allowance(self.asset_id, owner, spender).unwrap_or(0)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: DefaultAccountId,
            value: DefaultBalance,
        ) -> Result<()> {
            self.env().extension().transfer(self.asset_id, to, value)
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: DefaultAccountId,
            to: DefaultAccountId,
            value: DefaultBalance,
        ) -> Result<()> {
            self.env().extension().transfer_from(self.asset_id, from, to, value)
        }

        #[ink(message)]
        fn approve(
            &mut self,
            spender: DefaultAccountId,
            value: DefaultBalance,
        ) -> Result<()> {
            self.env().extension().approve(self.asset_id, spender, value)
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: DefaultAccountId,
            value: DefaultBalance,
        ) -> Result<()> {
            self.env().extension().increase_allowance(self.asset_id, spender, value)
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: DefaultAccountId,
            value: DefaultBalance,
        ) -> Result<()> {
            self.env().extension().decrease_allowance(self.asset_id, spender, value)
        }

        // Metadata interfaces

        #[ink(message)]
        fn token_name(&self) -> Result<Vec<u8>> {
            self.env().extension().token_name(self.asset_id)
        }

        #[ink(message)]
        fn token_symbol(&self) -> Result<Vec<u8>> {
            self.env().extension().token_symbol(self.asset_id)
        }

        #[ink(message)]
        fn token_decimals(&self) -> Result<u8> {
            self.env().extension().token_decimals(self.asset_id)
        }

        #[ink(message)]
        fn mint(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<()> {
            self.env().extension().mint(self.asset_id, to, value)
        }

        #[ink(message)]
        fn burn(&mut self, from: DefaultAccountId, value: DefaultBalance) -> Result<()> {
            self.env().extension().burn(self.asset_id, from, value)
        }
    }
    
}
