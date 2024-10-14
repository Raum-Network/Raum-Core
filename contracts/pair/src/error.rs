use soroban_sdk::contracterror;

use crate::pair_token::PairTokenError;



#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RaumFiPairError {
    Locked = 1,
    AlreadyInitialized = 2,
    Forbidden = 3,
    InsufficientLiquidityMinted = 4,
    InsufficientOutputAmount = 6,
    InsufficientLiquidity = 7,
    InvalidTo = 8,
    InsufficientInputAmount = 9,
    K = 10,
    Overflow = 11,
    CreatePairIdenticalTokens = 12,
    PairTokenError = 13,
    InvalidAmount = 14,
    InsufficientLiquidityBurned = 15,
}

