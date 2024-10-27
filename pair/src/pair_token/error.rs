use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PairTokenError {
    InsufficientBalance = 1,
    InsufficientAllowance = 2,
    InvalidAmount = 3,
}