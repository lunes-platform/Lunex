use ink::{
    prelude::vec::Vec,
    primitives::Hash,
};
use openbrush::{
    storage::Mapping,
    traits::AccountId
};
#[derive(Debug)]
#[openbrush::storage_item]
pub struct Data {
    pub fee_to: AccountId,
    pub fee_to_setter: AccountId,
    pub get_pair: Mapping<(AccountId, AccountId), AccountId>,
    pub all_pairs: Vec<AccountId>,
    pub pair_contract_code_hash: Hash,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            fee_to: [0u8; 32].into(),
            fee_to_setter: [0u8; 32].into(),
            get_pair: Default::default(),
            all_pairs: Vec::new(),
            pair_contract_code_hash: Default::default(),
        }
    }
}
