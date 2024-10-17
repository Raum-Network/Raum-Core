use core::clone;

use raumfi_library::RaumFiV2Library;
use soroban_sdk::{
    contract, contractimpl, Address, Env,  Symbol, Vec,
     symbol_short , token::Client as TokenClient, log
};
use raumfi_library::*;

use crate::factory_client::FactoryClient;
use crate::helper::*;

soroban_sdk::contractimport!(
    file = "D:/Raum-Core/target/wasm32-unknown-unknown/release/pair.wasm"
);
pub type PairClient<'a> = Client<'a>;

pub const FACTORY: Symbol = symbol_short!("Factory");
pub const NATIVE: Symbol = symbol_short!("Native");

#[contract]
pub struct RaumFiRouter;

#[contractimpl]
impl RaumFiRouter {
    pub fn initialize(env: Env, factory: Address, native: Address) {
        env.storage().instance().set(&FACTORY, &factory);
        env.storage().instance().set(&NATIVE, &native);
    }

    fn factory(env: &Env) -> Address {
        env.storage().instance().get(&FACTORY).unwrap()
    }

    fn native(env: &Env) -> Address {
        env.storage().instance().get(&NATIVE).unwrap()
    }

    fn ensure(env: &Env, deadline: u64) {
        if env.ledger().timestamp() > deadline {
            panic!("RaumFiRouter: Deadline Expired");
        }
    }

    fn get_native(env: &Env) -> Address {
        env.storage().instance().get(&NATIVE).unwrap()
    }

    fn _add_liquidity(
        env: &Env,
        token_a: &Address,
        token_b: &Address,
        amount_a_desired: i128,
        amount_b_desired: i128,
        amount_a_min: i128,
        amount_b_min: i128,
    ) -> (i128, i128) {
        let factory = Self::factory(env);
        let factory_client = FactoryClient::new(env, &factory);
        if factory_client.get_pair(token_a, token_b).is_none() {
            factory_client.create_new_pair(token_a, token_b);
        }
        let (token_0, token_1) = crate::helper::sort_tokens(token_a.clone(), token_b.clone()).unwrap();
        let pair = factory_client.get_pair(&token_0, &token_1).unwrap();
        let pair_client = PairClient::new(env, &pair);
        let (reserve_a, reserve_b) = pair_client.get_reserves();
        if reserve_a == 0 && reserve_b == 0 {
            (amount_a_desired, amount_b_desired)
        } else {
            let amount_b_optimal =  RaumFiV2Library::calculate_quote(env, amount_a_desired, reserve_a, reserve_b).unwrap();
            if amount_b_optimal <= amount_b_desired {
                if amount_b_optimal < amount_b_min {
                    panic!("RaumFiRouter: INSUFFICIENT_B_AMOUNT");
                }
                (amount_a_desired, amount_b_optimal)
            } else {
                let amount_a_optimal = RaumFiV2Library::calculate_quote(env , amount_b_desired, reserve_b, reserve_a).unwrap();
                assert!(amount_a_optimal <= amount_a_desired);
                if amount_a_optimal < amount_a_min {
                    panic!("RaumFiRouter: INSUFFICIENT_A_AMOUNT");
                }
                (amount_a_optimal, amount_b_desired)
            }
        }
    }

    pub fn add_liquidity(
        env: Env,
        token_a: Address,
        token_b: Address,
        amount_a_desired: i128,
        amount_b_desired: i128,
        amount_a_min: i128,
        amount_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> (i128, i128, i128) {
        Self::ensure(&env, deadline);
        to.require_auth();
        let (amount_a, amount_b) = Self::_add_liquidity(
            &env,
            &token_a,
            &token_b,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        );
        
        let pair = crate::helper::pair_for(env.clone(), Self::factory(&env), token_a.clone(), token_b.clone()).unwrap();
        let pair_client = PairClient::new(&env, &pair);
        TokenClient::new(&env, &token_a).transfer(&to, &pair, &amount_a);
        TokenClient::new(&env, &token_b).transfer(&to, &pair, &amount_b);
        let liquidity = pair_client.mint(&to);
        (amount_a, amount_b, liquidity)
    }


    pub fn remove_liquidity(
        env: Env,
        token_a: Address,
        token_b: Address,
        liquidity: i128,
        amount_a_min: i128,
        amount_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> (i128, i128) {
        Self::ensure(&env, deadline);
        to.require_auth();
        let factory_client = FactoryClient::new(&env, &Self::factory(&env));
        
        if factory_client.get_pair(&token_a, &token_b).is_none() {
            panic!("RaumFiRouter: PAIR_DOES_NOT_EXIST");
        }
        
        let pair = crate::helper::pair_for(env.clone(), Self::factory(&env).clone(), token_a.clone(), token_b.clone()).unwrap();
        let pair_client = PairClient::new(&env, &pair);
        pair_client.transfer(&to , &pair, &liquidity);
        let (amount0, amount1) = pair_client.burn(&to);
        let (token0, _) = crate::helper::sort_tokens(token_a.clone(), token_b.clone()).unwrap();
        let (amount_a, amount_b) = if token_a == token0 {
            (amount0, amount1)
        } else {
            (amount1, amount0)
        };
        if amount_a < amount_a_min {
            panic!("RaumFiRouter: INSUFFICIENT_A_AMOUNT");
        }
        if amount_b < amount_b_min {
            panic!("RaumFiRouter: INSUFFICIENT_B_AMOUNT");
        }
        (amount_a, amount_b)
    }

    pub fn quote(env: Env, amount_a: i128, reserve_a: i128, reserve_b: i128) -> i128 {
        let quote = RaumFiV2Library::calculate_quote(&env, amount_a, reserve_a, reserve_b).unwrap();
        log!(&env, "quote: {}", quote);
        quote
    }

    pub fn get_amount_out(env: Env, amount_in: i128, reserve_in: i128, reserve_out: i128) -> i128 {
        RaumFiV2Library::get_amount_out(&env, amount_in, reserve_in, reserve_out).unwrap()
    }

    pub fn get_amount_in(env: Env, amount_out: i128, reserve_in: i128, reserve_out: i128) -> i128 {
        RaumFiV2Library::get_amount_in(&env, amount_out, reserve_in, reserve_out).unwrap()
    }

    pub fn get_amounts_out(env: Env, amount_in: i128, path: Vec<Address>) -> Vec<i128> {
        RaumFiV2Library::get_amounts_out(&env, Self::factory(&env), amount_in, path).unwrap()
    }

    pub fn get_amounts_in(env: Env, amount_out: i128, path: Vec<Address>) -> Vec<i128> {
        RaumFiV2Library::get_amounts_in(&env, Self::factory(&env), amount_out, path).unwrap()
    }

    pub fn swap_exact_tokens_for_tokens(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,

        to: Address,
        deadline: u64,
    ) -> Vec<i128> {
        Self::ensure(&env, deadline);
        to.require_auth();
        let amounts = RaumFiV2Library::get_amounts_out(&env, Self::factory(&env), amount_in, path.clone()).unwrap();
        if amounts.get(amounts.len() - 1).unwrap() < amount_out_min {
            panic!("RaumFiRouter: INSUFFICIENT_OUTPUT_AMOUNT");
        }
        let pair = RaumFiV2Library::pair_for(&env, Self::factory(&env), path.clone().get(0).unwrap().clone(), path.clone().get(1).unwrap().clone()).unwrap();
        TokenClient::new(&env, &path.get(0).unwrap()).transfer(&to, &pair, &amounts.get(0).unwrap());
        Self::_swap(&env, &amounts, &path, &to);
        amounts
    }

    pub fn swap_tokens_for_exact_tokens(
        env: Env,
        amount_out: i128,
        amount_in_max: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128> {
        Self::ensure(&env, deadline);
        to.require_auth();

        let amounts = RaumFiV2Library::get_amounts_in(&env, Self::factory(&env), amount_out, path.clone()).unwrap();
        if amounts.get(0).unwrap() > amount_in_max {
            panic!("RaumFiRouter: EXCESSIVE_INPUT_AMOUNT");
        }
        let pair = RaumFiV2Library::pair_for(&env, Self::factory(&env), path.get(0).unwrap().clone(), path.get(1).unwrap().clone()).unwrap();
        TokenClient::new(&env, &path.get(0).unwrap()).transfer(&to, &pair, &amounts.get(0).unwrap());
        Self::_swap(&env, &amounts, &path, &to);
        amounts
    }

    fn _swap(env: &Env, amounts: &Vec<i128>, path: &Vec<Address>, _to: &Address) {
        for i in 0..path.len() - 1 {
            let (input, output) = (path.get(i).unwrap().clone(), path.get(i + 1).unwrap().clone());
            let (token0, _) = RaumFiV2Library::sort_tokens(input.clone(), output.clone()).unwrap();
            let amount_out = amounts.get(i + 1).unwrap();
            let (amount0_out, amount1_out) = if input == token0 {
                (0, amount_out)
            } else {
                (amount_out, 0)
            };
            let to = if i < path.len() - 2 {
                RaumFiV2Library::pair_for(env, Self::factory(env), output.clone(), path.get(i + 2).unwrap().clone()).unwrap()
            } else {
                _to.clone()
            };
            let pair = RaumFiV2Library::pair_for(env, Self::factory(env), input.clone(), output.clone()).unwrap();
            PairClient::new(env, &pair).swap(&amount0_out, &amount1_out, &to);
        }
    }
}
