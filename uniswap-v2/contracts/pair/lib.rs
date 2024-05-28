#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![warn(clippy::arithmetic_side_effects)]

#[openbrush::implementation(PSP22, Ownable)]
#[openbrush::contract]
pub mod pair {
    use ink::{
        codegen::{
            EmitEvent,
            Env,
        }
    };
    use openbrush::{
        contracts::{
            ownable::*,
            psp22::*,
            reentrancy_guard,
        },
        traits::Storage,
    };
    use uniswap_v2::impls::pair::{pair::*, data };
    use uniswap_v2::ensure;
    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: Balance,
        pub amount_1: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: Balance,
        pub amount_1: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0_in: Balance,
        pub amount_1_in: Balance,
        pub amount_0_out: Balance,
        pub amount_1_out: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Sync {
        reserve_0: Balance,
        reserve_1: Balance,
    }

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
    pub struct PairContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]
        pair: data::Data,
    }

    impl uniswap_v2::impls::pair::pair::Internal for PairContract {
        fn _emit_mint_event(&self, sender: AccountId, amount_0: Balance, amount_1: Balance) {
            self.env().emit_event(Mint {
                sender,
                amount_0,
                amount_1,
            })
        }

        fn _emit_burn_event(
            &self,
            sender: AccountId,
            amount_0: Balance,
            amount_1: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Burn {
                sender,
                amount_0,
                amount_1,
                to,
            })
        }

        fn _emit_swap_event(
            &self,
            sender: AccountId,
            amount_0_in: Balance,
            amount_1_in: Balance,
            amount_0_out: Balance,
            amount_1_out: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Swap {
                sender,
                amount_0_in,
                amount_1_in,
                amount_0_out,
                amount_1_out,
                to,
            })
        }

        fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance) {
            self.env().emit_event(Sync {
                reserve_0,
                reserve_1,
            })
        }

        fn _emit_transfer_event(&self, from: Option<AccountId>, to: Option<AccountId>, value: Balance) {
            self.env().emit_event(Transfer {
                from,
                to,
                value,
            })
        }

        fn _emit_approval_event(&self, owner: AccountId, spender: AccountId, value: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            })
        }

        fn _mint_to(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut new_balance = self.psp22.balances.get(&account).unwrap_or_default();
            new_balance += amount;
            self.psp22.balances.insert(&account, &new_balance);
            let mut supply = self.psp22.supply.get().unwrap_or_default();
            supply += amount;
            self.psp22.supply.set(&supply);
            self.env().emit_event(Transfer {
                from:None,
                to:account.into(),
                value:amount,
            });
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut from_balance = self.psp22.balances.get(&account).unwrap_or_default();
            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            from_balance -= amount;
            self.psp22.balances.insert(&account, &from_balance);
            let mut supply = self.psp22.supply.get().unwrap_or_default();
            supply -= amount;
            self.psp22.supply.set(&supply);
            self.env().emit_event(Transfer {
                from:account.into(),
                to:None,
                value:amount,
            });
            Ok(())
        }

        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            self.psp22.allowances.insert(&(&owner, &spender), &amount);
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
            Ok(())
        }

        
        fn _allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.psp22.allowances.get(&(&owner, &spender)).unwrap_or_default()
        }
        fn _balance_of (&self, account: AccountId) -> Balance {
            self.psp22.balances.get(&account).unwrap_or_default()
        }
        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let mut from_balance = self.psp22.balances.get(&from).unwrap_or_default();
            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);
            from_balance -= amount;
            self.psp22.balances.insert(&from, &from_balance);
            let mut to_balance = self.psp22.balances.get(&to).unwrap_or_default();
            to_balance += amount;
            self.psp22.balances.insert(&to, &to_balance);
            self.env().emit_event(Transfer {
                from:from.into(),
                to:to.into(),
                value:amount,
            });
            Ok(())
        }

    }

    impl Pair for PairContract {}
    
    impl PairContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            ownable::InternalImpl::_init_with_owner(&mut instance, caller);
            instance.pair.factory = caller;
            instance
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn initialize_works() {
            let mut pair = PairContract::new();
            let token_0 = AccountId::from([0x03; 32]);
            let token_1 = AccountId::from([0x04; 32]);
            assert_eq!(pair.initialize(token_0, token_1), Ok(()));
        }
    }
}
