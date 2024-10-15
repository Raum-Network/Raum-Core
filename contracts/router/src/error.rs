use soroban_sdk::{self, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RaumFiRouterError {
    /// RaumFiLibrary: insufficient amount 
    InsufficientAmount = 301,

    /// RaumFiLibrary: insufficient liquidity
    InsufficientLiquidity = 302,

    /// RaumFiLibrary: insufficient input amount
    InsufficientInputAmount = 303,

    /// RaumFiLibrary: insufficient output amount
    InsufficientOutputAmount = 304,

    /// RaumFiLibrary: invalid path
    InvalidPath = 305,

    /// RaumFiLibrary: token_a and token_b have identical addresses
    SortIdenticalTokens = 306,
}