#![no_std]

use soroban_sdk::{
    contract, contractimpl,
    Address, Env, Vec, 
};

mod library;
mod error;
mod interface;
#[cfg(test)]
mod test;

pub use library::*;
pub use interface::*;

pub trait RaumFiLibraryTrait {
    /// Sort two token addresses
    fn sort_tokens( token_a: Address, token_b: Address) -> Result<(Address, Address), RaumFiLibraryError>;
    
    /// Calculate the address for a pair
    fn pair_for(env: &Env, factory: Address, token_a: Address, token_b: Address) -> Result<Address, RaumFiLibraryError>;
    
    /// Fetch and sort the reserves for a pair of tokens
    fn get_reserves(env: &Env, factory: Address, token_a: Address, token_b: Address) -> Result<(i128, i128), RaumFiLibraryError>;
    
    /// Quote the amount of output tokens for a given input amount and reserves
    fn calculate_quote(env: &Env, amount_a: i128, reserve_a: i128, reserve_b: i128) -> Result<i128, RaumFiLibraryError>;
    
    /// Calculate the output amount for a swap
    fn get_amount_out(env: &Env, amount_in: i128, reserve_in: i128, reserve_out: i128) -> Result<i128, RaumFiLibraryError>;
    
    /// Calculate the input amount for a desired output
    fn get_amount_in(env: &Env, amount_out: i128, reserve_in: i128, reserve_out: i128) -> Result<i128, RaumFiLibraryError>;
    
    /// Calculate amounts out for a given input amount and path
    fn get_amounts_out(env: &Env, factory: Address, amount_in: i128, path: Vec<Address>) -> Result<Vec<i128>, RaumFiLibraryError>;
    
    /// Calculate amounts in for a desired output amount and path
    fn get_amounts_in(env: &Env, factory: Address, amount_out: i128, path: Vec<Address>) -> Result<Vec<i128>, RaumFiLibraryError>;
    
    /// Calculate the constant product 'k'
    fn calculate_k(reserve_a: i128, reserve_b: i128) -> Result<i128, RaumFiLibraryError>;
    
    /// Calculate optimal liquidity amounts
    fn optimal_liquidity(
        env: &Env,
        amount_a_desired: i128,
        amount_b_desired: i128,
        amount_a_min: i128,
        amount_b_min: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<(i128, i128), RaumFiLibraryError>;
    
    /// Calculate price impact of a swap
    fn calculate_price_impact(
        env: &Env,
        amount_in: i128,
        amount_out: i128,
        reserve_in: i128,
        reserve_out: i128,
    ) -> Result<i128, RaumFiLibraryError>;
    
    /// Calculate liquidity to be minted
    fn calculate_liquidity_minted(
        env: &Env,
        total_supply: i128,
        amount_a: i128,
        amount_b: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<i128, RaumFiLibraryError>;
    
    /// Calculate amounts to be returned when burning liquidity
    fn calculate_burn_amounts(
        env: &Env,
        liquidity: i128,
        total_supply: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<(i128, i128), RaumFiLibraryError>;
    
    /// Check if the constant product is maintained after a swap
    fn is_constant_product_maintained(
        env: &Env,
        reserve_a: i128,
        reserve_b: i128,
        new_reserve_a: i128,
        new_reserve_b: i128,
    ) -> bool;
}


#[contract]
pub struct RaumFiV2Library;

#[contractimpl]
impl RaumFiLibraryTrait for RaumFiV2Library {
/// Sort two token addresses
fn sort_tokens( token_a: Address, token_b: Address) -> Result<(Address, Address), RaumFiLibraryError>{
   sort_tokens(token_a, token_b)
}
    
/// Calculate the address for a pair
fn pair_for(env: &Env, factory: Address, token_a: Address, token_b: Address) -> Result<Address, RaumFiLibraryError>{
    pair_for(env, factory, token_a, token_b)
}

/// Fetch and sort the reserves for a pair of tokens
fn get_reserves(env: &Env, factory: Address, token_a: Address, token_b: Address) -> Result<(i128, i128), RaumFiLibraryError>{
    get_reserves(env, factory, token_a, token_b)
}

/// Quote the amount of output tokens for a given input amount and reserves
fn calculate_quote(env: &Env, amount_a: i128, reserve_a: i128, reserve_b: i128) -> Result<i128, RaumFiLibraryError>{
    calculate_quote(env, amount_a, reserve_a, reserve_b)
}

/// Calculate the output amount for a swap
fn get_amount_out(env: &Env, amount_in: i128, reserve_in: i128, reserve_out: i128) -> Result<i128, RaumFiLibraryError>{
    get_amount_out(env, amount_in, reserve_in, reserve_out)
}

/// Calculate the input amount for a desired output
fn get_amount_in(env: &Env, amount_out: i128, reserve_in: i128, reserve_out: i128) -> Result<i128, RaumFiLibraryError>{
    get_amount_in(env, amount_out, reserve_in, reserve_out)
}

/// Calculate amounts out for a given input amount and path
fn get_amounts_out(env: &Env, factory: Address, amount_in: i128, path: Vec<Address>) -> Result<Vec<i128>, RaumFiLibraryError>{
    get_amounts_out(env, factory, amount_in, path)
}

/// Calculate amounts in for a desired output amount and path
fn get_amounts_in(env: &Env, factory: Address, amount_out: i128, path: Vec<Address>) -> Result<Vec<i128>, RaumFiLibraryError>{
    get_amounts_in(env, factory, amount_out, path)
}

/// Calculate the constant product 'k'
fn calculate_k(reserve_a: i128, reserve_b: i128) -> Result<i128, RaumFiLibraryError>{
    calculate_k(reserve_a, reserve_b)
}

/// Calculate optimal liquidity amounts
fn optimal_liquidity(
    env: &Env,
    amount_a_desired: i128,
    amount_b_desired: i128,
    amount_a_min: i128,
    amount_b_min: i128,
    reserve_a: i128,
    reserve_b: i128,
) -> Result<(i128, i128), RaumFiLibraryError>{
    optimal_liquidity(env, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min, reserve_a, reserve_b)
}

/// Calculate price impact of a swap
fn calculate_price_impact(
    env: &Env,
    amount_in: i128,
    amount_out: i128,
    reserve_in: i128,
    reserve_out: i128,
) -> Result<i128, RaumFiLibraryError>{
    calculate_price_impact(env, amount_in, amount_out, reserve_in, reserve_out)
}

/// Calculate liquidity to be minted
fn calculate_liquidity_minted(
    env: &Env,
    total_supply: i128,
    amount_a: i128,
    amount_b: i128,
    reserve_a: i128,
    reserve_b: i128,
) -> Result<i128, RaumFiLibraryError>{
    calculate_liquidity_minted(env, total_supply, amount_a, amount_b, reserve_a, reserve_b)
}

/// Calculate amounts to be returned when burning liquidity
fn calculate_burn_amounts(
    env: &Env,
    liquidity: i128,
    total_supply: i128,
    reserve_a: i128,
    reserve_b: i128,
) -> Result<(i128, i128), RaumFiLibraryError>{
    calculate_burn_amounts(env, liquidity, total_supply, reserve_a, reserve_b)
}

/// Check if the constant product is maintained after a swap
fn is_constant_product_maintained(
    env: &Env,
    reserve_a: i128,
    reserve_b: i128,
    new_reserve_a: i128,
    new_reserve_b: i128,
) -> bool{
    is_constant_product_maintained(env, reserve_a, reserve_b, new_reserve_a, new_reserve_b)
}
}
