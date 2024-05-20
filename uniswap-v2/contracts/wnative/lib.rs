#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[openbrush::implementation(PSP22, PSP22Metadata)]
#[openbrush::contract]
pub mod wnative {

    use openbrush::{
        traits::{
            Storage,
            String,
        },
    };
    use uniswap_v2::impls::wnative::*;

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
    #[derive(Default, Storage)]
    pub struct WnativeContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl Wnative for WnativeContract {}


    impl WnativeContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance.metadata.name.set(&Some(String::from("Lunes Nightly")));
            instance.metadata.symbol.set(&Some(String::from("LUNES")));
            instance.metadata.decimals.set(&8u8);
            instance
        }
        #[ink(message)]
        pub fn transfer_event(
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
        pub fn approval_event(&self, owner: AccountId, spender: AccountId, amount: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
        }
    }

    #[cfg(test)]
    mod tests {
      
    }
}
