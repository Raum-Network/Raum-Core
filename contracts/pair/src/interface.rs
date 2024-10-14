
use soroban_sdk::{contractclient, contractspecfn, Address, Env , BytesN};


pub use crate::factory_error::FactoryError;
pub struct Spec;

/// Interface for RaumFiFactory
#[contractspecfn(name = "Spec", export = false)]
#[contractclient(name = "RaumFiFactoryClient")]

/// Trait defining the interface for a RaumFi Factory contract.
pub trait RaumFiFactoryTrait {

    fn fee_to(e: Env) -> Result<Address, FactoryError>;


    fn fee_to_setter(e: Env) -> Result<Address, FactoryError>;


    fn fees_enabled(e: Env) -> Result<bool, FactoryError>;


    fn all_pairs_length(e: Env) -> Result<u32, FactoryError>;


    fn get_pair(e: Env, token_a: Address, token_b: Address) -> Result<Address, FactoryError>;


    fn all_pairs(e: Env, n: u32) -> Result<Address, FactoryError>;

    fn get_fee_to(e: Env) -> Result<Address, FactoryError>;


    fn pair_exists(e: Env, token_a: Address, token_b: Address) -> Result<bool, FactoryError>;

    fn initialize(e: Env, setter: Address, pair_wasm_hash: BytesN<32>) -> Result<(), FactoryError>;

    fn set_fee_to(e: Env, to: Address)-> Result<(), FactoryError>;

    fn set_fee_to_setter(e: Env, new_setter: Address)-> Result<(), FactoryError>;

    fn set_fees_enabled(e: Env, is_enabled: bool)-> Result<(), FactoryError>;

    fn create_pair(e: Env, token_a: Address, token_b: Address) -> Result<Address, FactoryError>;
}
