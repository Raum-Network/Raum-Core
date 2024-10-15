use raumfi_library::RaumFiV2Library;
use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, BytesN, Symbol, Vec, Map,
    IntoVal, TryFromVal, ConversionError, symbol_short , token::Client as TokenClient
};
use raumfi_library::*;

use crate::factory_client::{ self, FactoryClient};
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
        if !factory_client.pair_exists(token_a, token_b) {
            factory_client.create_new_pair(token_a, token_b);
        }
        let (token_0,token_1) = sort_tokens(token_a.clone(), token_b.clone()).unwrap();
        let pair = factory_client.get_pair(&token_0, &token_1).unwrap();
        let pair_client = PairClient::new(env, &pair);
        let (reserve_a, reserve_b) = pair_client.get_reserves();
        if reserve_a == 0 && reserve_b == 0 {
            (amount_a_desired, amount_b_desired)
        } else {
            let amount_b_optimal =  RaumFiV2Library::quote(env, amount_a_desired, reserve_a, reserve_b).unwrap();
            if amount_b_optimal <= amount_b_desired {
                if amount_b_optimal < amount_b_min {
                    panic!("RaumFiRouter: INSUFFICIENT_B_AMOUNT");
                }
                (amount_a_desired, amount_b_optimal)
            } else {
                let amount_a_optimal = RaumFiV2Library::quote(env , amount_b_desired, reserve_b, reserve_a).unwrap();
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
        
        let pair = pair_for(env.clone(), Self::factory(&env), token_a.clone(), token_b.clone()).unwrap();
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
        let factory_client = FactoryClient::new(&env, &Self::factory(&env));
        if !factory_client.pair_exists(&token_a, &token_b) {
            panic!("RaumFiRouter: PAIR_DOES_NOT_EXIST");
        }
        let pair = pair_for(env.clone(), factory_client.address, token_a.clone(), token_b.clone()).unwrap();
        TokenClient::new(&env, &pair).transfer(&to, &pair, &liquidity);
        let (amount0, amount1) = PairClient::new(&env, &pair).burn(&to);
        let (token0, _) = sort_tokens(token_a.clone(), token_b.clone()).unwrap();
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
        RaumFiV2Library::quote(&env, amount_a, reserve_a, reserve_b).unwrap()
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
        let amounts = RaumFiV2Library::get_amounts_out(&env, &Self::Factory(&env), amount_in, &path);
        if amounts[amounts.len() - 1] < amount_out_min {
            panic!("UniswapV2Router: INSUFFICIENT_OUTPUT_AMOUNT");
        }
        let pair = RaumFiV2Library::pair_for(&env, &Self::Factory(&env), &path[0], &path[1]);
        TransferHelper::safe_transfer_from(&env, &path[0], &env.current_contract_address(), &pair, amounts[0]);
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
        let amounts = RaumFiV2Library::get_amounts_in(&env, &Self::Factory(&env), amount_out, &path);
        if amounts[0] > amount_in_max {
            panic!("UniswapV2Router: EXCESSIVE_INPUT_AMOUNT");
        }
        let pair = RaumFiV2Library::pair_for(&env, &Self::Factory(&env), &path[0], &path[1]);
        TransferHelper::safe_transfer_from(&env, &path[0], &env.current_contract_address(), &pair, amounts[0]);
        Self::_swap(&env, &amounts, &path, &to);
        amounts
    }

    pub fn swap_exact_eth_for_tokens(
        env: Env,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128> {
        Self::ensure(&env, deadline);
        let Native = Self::Native(&env);
        if path[0] != Native {
            panic!("UniswapV2Router: INVALID_PATH");
        }
        let amounts = RaumFiV2Library::get_amounts_out(&env, &Self::Factory(&env), env.attached_value(), &path);
        if amounts[amounts.len() - 1] < amount_out_min {
            panic!("UniswapV2Router: INSUFFICIENT_OUTPUT_AMOUNT");
        }
        INativeClient::new(&env, &Native).deposit(&env.current_contract_address(), env.attached_value());
        assert!(INativeClient::new(&env, &Native).transfer(&RaumFiV2Library::pair_for(&env, &Self::Factory(&env), &path[0], &path[1]), amounts[0]));
        Self::_swap(&env, &amounts, &path, &to);
        amounts
    }

    pub fn swap_tokens_for_exact_eth(
        env: Env,
        amount_out: i128,
        amount_in_max: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128> {
        Self::ensure(&env, deadline);
        let Native = Self::Native(&env);
        if path[path.len() - 1] != Native {
            panic!("UniswapV2Router: INVALID_PATH");
        }
        let amounts = RaumFiV2Library::get_amounts_in(&env, &Self::Factory(&env), amount_out, &path);
        if amounts[0] > amount_in_max {
            panic!("UniswapV2Router: EXCESSIVE_INPUT_AMOUNT");
        }
        let pair = RaumFiV2Library::pair_for(&env, &Self::Factory(&env), &path[0], &path[1]);
        TransferHelper::safe_transfer_from(&env, &path[0], &env.current_contract_address(), &pair, amounts[0]);
        Self::_swap(&env, &amounts, &path, &env.current_contract_address());
        INativeClient::new(&env, &Native).withdraw(amounts[amounts.len() - 1]);
        TransferHelper::safe_transfer_eth(&env, &to, amounts[amounts.len() - 1]);
        amounts
    }

    pub fn swap_exact_tokens_for_eth(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128> {
        Self::ensure(&env, deadline);
        let Native = Self::Native(&env);
        if path[path.len() - 1] != Native {
            panic!("UniswapV2Router: INVALID_PATH");
        }
        let amounts = RaumFiV2Library::get_amounts_out(&env, &Self::Factory(&env), amount_in, &path);
        if amounts[amounts.len() - 1] < amount_out_min {
            panic!("UniswapV2Router: INSUFFICIENT_OUTPUT_AMOUNT");
        }
        let pair = RaumFiV2Library::pair_for(&env, &Self::Factory(&env), &path[0], &path[1]);
        TransferHelper::safe_transfer_from(&env, &path[0], &env.current_contract_address(), &pair, amounts[0]);
        Self::_swap(&env, &amounts, &path, &env.current_contract_address());
        INativeClient::new(&env, &Native).withdraw(amounts[amounts.len() - 1]);
        TransferHelper::safe_transfer_eth(&env, &to, amounts[amounts.len() - 1]);
        amounts
    }

    pub fn swap_eth_for_exact_tokens(
        env: Env,
        amount_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<i128> {
        Self::ensure(&env, deadline);
        let Native = Self::Native(&env);
        if path[0] != Native {
            panic!("UniswapV2Router: INVALID_PATH");
        }
        let amounts = RaumFiV2Library::get_amounts_in(&env, &Self::Factory(&env), amount_out, &path);
        if amounts[0] > env.attached_value() {
            panic!("UniswapV2Router: EXCESSIVE_INPUT_AMOUNT");
        }
        INativeClient::new(&env, &Native).deposit(&env.current_contract_address(), amounts[0]);
        assert!(INativeClient::new(&env, &Native).transfer(&RaumFiV2Library::pair_for(&env, &Self::Factory(&env), &path[0], &path[1]), amounts[0]));
        Self::_swap(&env, &amounts, &path, &to);
        if env.attached_value() > amounts[0] {
            TransferHelper::safe_transfer_eth(&env, &env.current_contract_address(), env.attached_value() - amounts[0]);
        }
        amounts
    }

    fn _swap(env: &Env, amounts: &Vec<i128>, path: &Vec<Address>, _to: &Address) {
        for i in 0..path.len() - 1 {
            let (input, output) = (path[i].clone(), path[i + 1].clone());
            let (token0, _) = RaumFiV2Library::sort_tokens(&input, &output);
            let amount_out = amounts[i + 1];
            let (amount0_out, amount1_out) = if input == token0 {
                (0, amount_out)
            } else {
                (amount_out, 0)
            };
            let to = if i < path.len() - 2 {
                RaumFiV2Library::pair_for(env, &Self::Factory(env), &output, &path[i + 2])
            } else {
                _to.clone()
            };
            let pair = RaumFiV2Library::pair_for(env, &Self::Factory(env), &input, &output);
            PairClient::new(env, &pair).swap(&amount0_out, &amount1_out, &to, &Vec::new());
        }
    }
}