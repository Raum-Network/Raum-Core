use soroban_sdk::{self, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RaumFiFactoryError {

    NotInitialized = 1,
    CreatePairIdenticalTokens = 2,
    CreatePairAlreadyExists = 3,
    InitializeAlreadyInitialized = 4,
    PairDoesNotExist = 5,
    IndexDoesNotExist = 6,
    PairAlreadyExists = 7,
}

