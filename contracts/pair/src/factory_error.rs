use soroban_sdk::{self, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    /// RaumFiFactory: not yet initialized
    NotInitialized = 201,

    /// RaumFiFactory: token_a and token_b have identical addresses
    CreatePairIdenticalTokens = 202,
    /// RaumFiFactory: pair already exists between token_a and token_b
    CreatePairAlreadyExists = 203,

    /// RaumFiFactory: already initialized
    InitializeAlreadyInitialized = 204,

    /// RaumFiFactory: pair does not exist
    PairDoesNotExist = 205,

    /// RaumFiFactory: index does not exist
    IndexDoesNotExist = 206,
}

