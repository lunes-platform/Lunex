use openbrush::traits::{AccountId, Balance};


#[derive(Debug)]
#[openbrush::storage_item]
pub struct Data {
    pub factory: AccountId,
    pub wnative: AccountId,
    pub fee: Balance,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            factory: [0u8; 32].into(),
            wnative: [0u8; 32].into(),
            fee:997u128,
        }
    }
}
