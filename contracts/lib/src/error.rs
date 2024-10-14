use soroban_sdk::{self, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RaumFiLibraryError {

    NotInitialized = 1,
    CreatePairIdenticalTokens = 2,
    CreatePairAlreadyExists = 3,
    InitializeAlreadyInitialized = 4,
    PairDoesNotExist = 5,
    IndexDoesNotExist = 6,
    IdenticalAddresses = 7,
    InsufficientLiquidity = 8,
    InsufficientAmount = 9,
    InvalidPath = 10,
    InsufficientLiquidityMinted = 11,
    InsufficientTotalSupply = 12,
    InsufficientLiquidityBurned = 13,
    InsufficientBAmount = 14,
    InsufficientAAmount = 15,
    Overflow = 16,
    DivisionByZero = 17,
}

