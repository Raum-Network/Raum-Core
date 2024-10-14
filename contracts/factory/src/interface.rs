use soroban_sdk::{contractclient, contractspecfn, Address, Env , BytesN};


pub use crate::error::RaumFiFactoryError;
pub struct Spec;

/// Interface for RaumFiFactory
#[contractspecfn(name = "Spec", export = false)]
#[contractclient(name = "RaumFiFactoryClient")]

pub trait RaumFiFactoryInterface {
    /// Initialize the factory with the fee_to_setter address
    fn initialize(env: Env, fee_to_setter: Address , pair_wasm_hash: BytesN<32>);

    /// Get the number of pairs created by the factory
    fn all_pairs_length(env: Env) -> u32;

    /// Create a new pair for the given tokens
    // fn create_pair(env: Env, token_a: Address, token_b: Address) -> Address;

    /// Get the address of the pair for the given tokens
    fn get_pair(env: Env, token_a: Address, token_b: Address) -> Option<Address>;

    /// Set the address that receives fees
    fn set_fee_to(env: Env, fee_to: Address);

    /// Get the address that receives fees
    fn get_fee_to(env: Env) -> Address;

    /// Set the address that can change the fee recipient
    fn set_fee_to_setter(env: Env, fee_to_setter: Address);

    fn create_new_pair(env: Env, token_a: Address, token_b: Address) -> Address;

    fn pair_exists(env: Env, token0: Address, token1: Address) -> bool;
}