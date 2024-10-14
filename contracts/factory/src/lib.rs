#![no_std]

#[cfg(test)]
mod test;

mod error;
mod interface;

pub use crate::error::RaumFiFactoryError;
use interface::RaumFiFactoryInterface;

use soroban_sdk::{
    contract, contracterror, contractimpl,contractimport ,contracttype, symbol_short, vec, xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal, Map, Symbol, Vec
};

mod pair {
    soroban_sdk::contractimport!(file = "D:/RaumFiV2/RaumFiV2/target/wasm32-unknown-unknown/release/pair.wasm");
    pub type PairClient<'a> = Client<'a>;
}

use pair::PairClient;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    IdenticalAddresses = 1,
    ZeroAddress = 2,
    PairExists = 3,
    Forbidden = 4,
}

#[contracttype]
#[derive(Clone)]
 pub enum DataKey {
    Pair(Address, Address),
}

#[contracttype]
#[derive(Clone)]
pub struct Pair(Address, Address);

impl Pair {
    pub fn new(a: Address, b: Address) -> Result<Self, RaumFiFactoryError> {
        if a == b {
            return Err(RaumFiFactoryError::CreatePairIdenticalTokens);
        }
        if a < b {
            Ok(Pair(a, b))
        } else {
            Ok(Pair(b, a))
        }
    }

    pub fn token_0(&self) -> &Address {
        &self.0
    }

    pub fn token_1(&self) -> &Address {
        &self.1
    }
}

pub trait RaumFiFactoryTrait {
    fn initialize(env: Env, fee_to_setter: Address, pair_wasm_hash: BytesN<32>);
    fn all_pairs_length(env: Env) -> u32;
    fn get_pair(env: Env, token_a: Address, token_b: Address) -> Option<Address>;
    fn set_fee_to(env: Env, fee_to: Address);
    fn set_fee_to_setter(env: Env, fee_to_setter: Address);
    fn pair_exists(env: &Env, token0: &Address, token1: &Address) -> bool;
} 

pub struct RaumFiV2Factory;

#[contract]
pub struct FactoryContract;

fn sort_tokens(token_a: &Address, token_b: &Address) -> (Address, Address) {
    if token_a < token_b {
        (token_a.clone(), token_b.clone())
    } else {
        (token_b.clone(), token_a.clone())
    }
}

#[contractimpl]
impl RaumFiFactoryInterface for FactoryContract  {
    fn initialize(env: Env, fee_to_setter: Address, pair_wasm_hash: BytesN<32>) {
        env.storage().persistent().set(&symbol_short!("feesetter"), &fee_to_setter);
        env.storage().persistent().set(&symbol_short!("feeto"), &fee_to_setter);
        env.storage().persistent().set(&symbol_short!("pair_hash"), &pair_wasm_hash);
        env.storage().persistent().set(&symbol_short!("pairs"), &0u32);
    }

    fn all_pairs_length(env: Env) -> u32 {
        env.storage().persistent().get(&symbol_short!("pairs")).unwrap_or(0)
    }

    fn get_pair(env: Env, token_a: Address, token_b: Address) -> Option<Address> {
        let (token0, token1) = sort_tokens(&token_a, &token_b);
        let key = DataKey::Pair(token0, token1);
        env.storage().instance().get(&key)
    }

    fn pair_exists(env: Env, token0: Address, token1: Address) -> bool {
        // Access storage directly without wrapping in a contract context
        env.storage().persistent().has(&DataKey::Pair(token0.clone(), token1.clone()))
    }
    

    fn set_fee_to(env: Env, fee_to: Address) {
        let setter: Address = env.storage().persistent().get(&symbol_short!("feesetter")).unwrap();
        setter.require_auth();
        env.storage().persistent().set(&symbol_short!("feeto"), &fee_to);
    }

    fn get_fee_to(env: Env) -> Address {
        env.storage().persistent().get(&symbol_short!("feeto")).unwrap()
    }

    fn set_fee_to_setter(env: Env, fee_to_setter: Address) {
        let current_fee_to_setter: Address = env.storage().persistent().get(&symbol_short!("feesetter")).unwrap();
        current_fee_to_setter.require_auth();
        env.storage().persistent().set(&symbol_short!("feesetter"), &fee_to_setter);
    }

    fn create_new_pair(env: Env, token_a: Address, token_b: Address) -> Address {
        RaumFiV2Factory::create_pair(&env, &token_a, &token_b)
    }
}

impl RaumFiV2Factory {

    fn set_fee_to(env: &Env, fee_to: &Address) {
        let setter: Address = env.storage().persistent().get(&symbol_short!("feesetter")).unwrap();
        setter.require_auth();
        env.storage().persistent().set(&symbol_short!("feeto"), fee_to);
    }

    fn set_fee_to_setter(env: &Env, fee_to_setter: &Address) {
        let current_fee_to_setter: Address = env.storage().persistent().get(&symbol_short!("feesetter")).unwrap();
        current_fee_to_setter.require_auth();
        env.storage().persistent().set(&symbol_short!("feesetter"), fee_to_setter);
    }

    fn pair_exists(env: Env, token0: Address, token1: Address) -> bool {
        // Access storage directly without wrapping in a contract context
        env.storage().instance().has(&DataKey::Pair(token0.clone(), token1.clone()))
    }

    fn create_pair(env: &Env, token_a: &Address, token_b: &Address) -> Address {
        if token_a == token_b {
            panic!("{}", Error::IdenticalAddresses as u32);
        }
        let (token0, token1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
       
        let key = DataKey::Pair(token0.clone(), token1.clone());
        let pair_exists = if env.storage().instance().has(&key) {
            true
        } else {
            false
        };

        if pair_exists == true {
            panic!("{}", Error::PairExists as u32);
        }

        // Create pair contract
        let mut salt = Bytes::new(env);
        salt.append(&token0.to_xdr(env));
        salt.append(&token1.to_xdr(env));
        let salt = env.crypto().sha256(&salt);
        let pair_wasm_hash: BytesN<32> = env.storage().persistent().get(&symbol_short!("pair_hash"))
        .unwrap();
        let pair: Address = env.deployer().with_current_contract(salt).deploy(pair_wasm_hash);
        // Initialize pair
        PairClient::new(&env, &pair).initialize(
           
            &token0, 
            &token1,
            &env.current_contract_address(),
        );

        let topics = (Symbol::new(env, "pairmade"), token_a.clone(), token_b.clone());
        env.events().publish(topics, pair.clone());
        env.storage().instance().set(&DataKey::Pair(token_a.clone(), token_b.clone()), &pair);
        let current_pairs = env.storage().persistent().get(&symbol_short!("pairs")).unwrap_or(0u32);
        env.storage().persistent().set(&symbol_short!("pairs"), &(current_pairs + 1));

        pair
    }

    // fn get_pair(env: &Env, token_a: &Address, token_b: &Address) -> Option<Address> {
    //     let (token0, token1) = sort_tokens(&token_a, &token_b);
    //     let key = DataKey::Pair(token0, token1);
    //     env.storage().instance().get(&key)
    // }
}
