#![cfg_attr(not(feature = "std"), no_std, no_main)]
use ink::{
    env::Environment,
    prelude::vec::Vec,
};
use openbrush::traits::String;
type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[ink::chain_extension]
pub trait Psp22Extension {
    type ErrorCode = Psp22ErrorExtension;

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
pub enum Psp22ErrorExtension {
    TotalSupplyFailed,
    BalanceOfFailed,
    AllowanceFailed,
    TransferFailed,
    TransferFromFailed,
    ApproveFailed,
    BalanceNoAllocated,
    CallerNotAllowed,
    BalanceTooLow,
    BalanceTooHigh,
    BalanceNotZero,
    InconsistentState,
    InsufficientAllowance,
    IncreaseAllowanceFailed,
    InsufficientBalance,
    DecreaseAllowanceFailed,
    TokenNameFailed,
    TokenSymbolFailed,
    TokenDecimalsFailed,
    MintFailed,
    BurnFailed,
    Custom(String),
}
impl Psp22ErrorExtension {
    pub fn as_str(&self) -> String {
        match self {
            Psp22ErrorExtension::TotalSupplyFailed => String::from("TotalSupplyFailed"),
            Psp22ErrorExtension::BalanceOfFailed => String::from("BalanceOfFailed"),
            Psp22ErrorExtension::AllowanceFailed => String::from("AllowanceFailed"),
            Psp22ErrorExtension::TransferFailed => String::from("TransferFailed"),
            Psp22ErrorExtension::TransferFromFailed => String::from("TransferFromFailed"),
            Psp22ErrorExtension::ApproveFailed => String::from("ApproveFailed"),
            Psp22ErrorExtension::BalanceNoAllocated => String::from("BalanceNoAllocated"),
            Psp22ErrorExtension::CallerNotAllowed => String::from("CallerNotAllowed"),
            Psp22ErrorExtension::BalanceTooLow => String::from("BalanceTooLow"),
            Psp22ErrorExtension::BalanceTooHigh => String::from("BalanceTooHigh"),
            Psp22ErrorExtension::BalanceNotZero => String::from("BalanceNotZero"),
            Psp22ErrorExtension::InconsistentState => String::from("InconsistentState"),
            Psp22ErrorExtension::InsufficientAllowance => String::from("InsufficientAllowance"),
            Psp22ErrorExtension::InsufficientBalance => String::from("InsufficientBalance"),
            Psp22ErrorExtension::IncreaseAllowanceFailed => String::from("IncreaseAllowanceFailed"),
            Psp22ErrorExtension::DecreaseAllowanceFailed => String::from("DecreaseAllowanceFailed"),
            Psp22ErrorExtension::TokenNameFailed => String::from("TokenNameFailed"),
            Psp22ErrorExtension::TokenSymbolFailed => String::from("TokenSymbolFailed"),
            Psp22ErrorExtension::TokenDecimalsFailed => String::from("TokenDecimalsFailed"),
            Psp22ErrorExtension::Custom(msg) => msg.clone(),
            Psp22ErrorExtension::MintFailed => String::from("MintFailed"),
            Psp22ErrorExtension::BurnFailed => String::from("BurnFailed"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Psp22ErrorExtension>;

impl From<scale::Error> for Psp22ErrorExtension {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink::env::chain_extension::FromStatusCode for Psp22ErrorExtension {
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
    use crate::{DefaultAccountId, DefaultBalance};
    use openbrush::contracts::psp22::PSP22Error;
    #[ink::trait_definition]
    pub trait PSP22 {
        
        #[ink(message)]
        fn total_supply(&self) -> DefaultBalance;

        #[ink(message)]
        fn balance_of(&self, owner: DefaultAccountId) -> DefaultBalance;

        #[ink(message)]
        fn allowance(&self, owner: DefaultAccountId, spender: DefaultAccountId) -> DefaultBalance;

        #[ink(message)]
        fn transfer(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<(), PSP22Error>;

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
}


#[ink::contract(env = crate::CustomEnvironment)]
mod token {
    use openbrush::contracts::psp22::PSP22Error;
    use super::{
        psp22::PSP22,
        Psp22ErrorExtension,
        Vec,
        DefaultAccountId,
        DefaultBalance,
    };
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }
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
        #[ink(message)]
        pub fn _emit_transfer_event(
            &self,
            from: Option<AccountId>,
            to: Option<AccountId>,
            amount: Balance,
        ) {
            self.env().emit_event(Transfer {
                from,
                to,
                value: amount,
            });
        }
        #[ink(message)]
        pub fn _emit_approval_event(&self, owner: AccountId, spender: AccountId, amount: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
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
        fn transfer(&mut self, to: DefaultAccountId, value: DefaultBalance) -> Result<(), PSP22Error>{
            let resp = self.env().extension().transfer(self.asset_id, to, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::TransferFailed.as_str()));
            }
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(&mut self, from: DefaultAccountId, to: AccountId, value: Balance) -> Result<(),PSP22Error>{
            let resp = self.env().extension().transfer_from(self.asset_id, from, to, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::TransferFromFailed.as_str()));
            }
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: DefaultAccountId, value: Balance) -> Result<(),PSP22Error>{
            let resp = self.env().extension().approve(self.asset_id, spender, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::ApproveFailed.as_str()));
            }
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(&mut self, spender: DefaultAccountId, value: Balance) -> Result<(),PSP22Error>{
            let resp = self.env().extension().increase_allowance(self.asset_id, spender, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::IncreaseAllowanceFailed.as_str()));
            }
            Ok(())
        }

        #[ink(message)]
        fn decrease_allowance(&mut self, spender: DefaultAccountId, value: Balance) -> Result<(),PSP22Error>{
            let resp = self.env().extension().decrease_allowance(self.asset_id, spender, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::DecreaseAllowanceFailed.as_str()));
            }
            Ok(())
        }

        // Metadata interfaces

        #[ink(message)]
        fn token_name(&self) -> Result<Vec<u8>,PSP22Error>{
            self.env().extension().token_name(self.asset_id).map_err(|_|PSP22Error::Custom(Psp22ErrorExtension::TokenNameFailed.as_str()))
        }

        #[ink(message)]
        fn token_symbol(&self) -> Result<Vec<u8>,PSP22Error>{
            self.env().extension().token_symbol(self.asset_id).map_err(|_|PSP22Error::Custom(Psp22ErrorExtension::TokenSymbolFailed.as_str()))
        }

        #[ink(message)]
        fn token_decimals(&self) -> Result<u8,PSP22Error>{
            self.env().extension().token_decimals(self.asset_id).map_err(|_|PSP22Error::Custom(Psp22ErrorExtension::TokenDecimalsFailed.as_str()))
        }

        #[ink(message)]
        fn mint(&mut self, to: AccountId, value: Balance) -> Result<(),PSP22Error>{
            let resp = self.env().extension().mint(self.asset_id, to, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::MintFailed.as_str()));
            }
            Ok(())
        }

        #[ink(message)]
        fn burn(&mut self, from: AccountId, value: Balance) -> Result<(),PSP22Error>{
            let resp = self.env().extension().burn(self.asset_id, from, value);
            if resp.is_err() {
                return Err(PSP22Error::Custom(Psp22ErrorExtension::BurnFailed.as_str()));
            }
            Ok(())
        }
    }
    
}
