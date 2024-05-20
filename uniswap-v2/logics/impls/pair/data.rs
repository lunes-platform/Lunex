use crate::traits::types::WrappedU256;
use openbrush::traits::{
    AccountId,
    Balance,
    Timestamp
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::storage_item]
pub struct Data {
    pub factory: AccountId,
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub reserve_0: Balance,
    pub reserve_1: Balance,
    pub block_timestamp_last: Timestamp,
    pub price_0_cumulative_last: WrappedU256,
    pub price_1_cumulative_last: WrappedU256,
    pub k_last: WrappedU256,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            factory: [0u8; 32].into(),
            token_0: [0u8; 32].into(),
            token_1: [0u8; 32].into(),
            reserve_0: 0,
            reserve_1: 0,
            block_timestamp_last: 0,
            price_0_cumulative_last: Default::default(),
            price_1_cumulative_last: Default::default(),
            k_last: Default::default(),
        }
    }
}
