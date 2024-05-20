use openbrush::traits::AccountId;


#[derive(Debug)]
#[openbrush::storage_item]
pub struct Data {
    pub factory: AccountId,
    pub wnative: AccountId,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            factory: [0u8; 32].into(),
            wnative: [0u8; 32].into(),
        }
    }
}
