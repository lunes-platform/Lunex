use openbrush::traits::String;



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
