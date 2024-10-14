#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{log, symbol_short, token, vec, Address, BytesN, Env};
use crate::library::{RaumFiV2LibraryClient  , RaumFiV2Library};
// Mock PairClient for testing


use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &sac),
        token::StellarAssetClient::new(e, &sac),
    )
}

fn pair_token_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "D:/RaumFiV2/RaumFiV2/target/wasm32-unknown-unknown/release/pair.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

pub mod pair {
    soroban_sdk::contractimport!(file = "D:/RaumFiV2/RaumFiV2/target/wasm32-unknown-unknown/release/pair.wasm");
    pub type PairClient<'a> = Client<'a>;
}
use pair::{PairClient, DataKey};

pub mod factory {
    soroban_sdk::contractimport!(file = "D:/RaumFiV2/RaumFiV2/target/wasm32-unknown-unknown/release/raumfi_factory.wasm");
    pub type FactoryClient<'a> = Client<'a>;
}
use factory::FactoryClient;

fn create_factory_contract<'a>(e: & Env, setter: & Address,pair_wasm_hash: & BytesN<32>) -> FactoryClient<'a> {
    let factory_address = &e.register_contract_wasm(None, factory::WASM);
    let factory = FactoryClient::new(e, factory_address);
    factory.initialize(&setter, pair_wasm_hash);
    factory
}

fn create_test_contract(env: &Env) -> RaumFiV2LibraryClient<'static> {
    let contract_id = env.register_contract(None, RaumFiV2Library{});
    let client = RaumFiV2LibraryClient::new(env, &contract_id);
    client
    
}

#[test]
fn test_sort_tokens() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token_a , token_a_client) = create_token_contract(&env, &admin);
    let (token_b , token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);

    let (token0, token1) = client.sort_tokens(&token_a_client.address, &token_b_client.address);

    assert!(token0 < token1);
    assert!((token0 == token_a_client.address && token1 == token_b_client.address) || (token0 == token_b_client.address && token1 == token_a_client.address));
}

#[test]
fn test_pair_for() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token_a , token_a_client) = create_token_contract(&env, &admin);
    let (token_b , token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);

    let pair_address = client.pair_for(&factory_client.address, &token_a_client.address, &token_b_client.address);

    assert!(pair_address != Address::generate(&env));
}

#[test]
fn test_get_reserves() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token_a , token_a_client) = create_token_contract(&env, &admin);
    let (token_b , token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);

    // Deploy mock pair contract
    let pair_address: Address = client.pair_for(&factory_client.address, &token_a.address, &token_b.address);
    let pair_client = PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a.address, &token_b.address, &factory_client.address);
    env.as_contract(&pair_address, || {
        env.storage().instance().set(&DataKey::Reserve0, &1000);
        env.storage().instance().set(&DataKey::Reserve1, &2000);
    });

    let (reserve_a, reserve_b) = client.get_reserves(&factory_client.address, &token_a_client.address, &token_b_client.address);

    assert_eq!(reserve_a, 1000);
    assert_eq!(reserve_b, 2000);
}

// #[test]
// fn test_quote() {
//     let env = create_test_env();
//     let amount_a = 100;
//     let reserve_a = 1000;
//     let reserve_b = 2000;

//     let amount_b = RaumFiV2Library::quote(&env, amount_a, reserve_a, reserve_b).unwrap();

//     assert_eq!(amount_b, 200);
// }

// #[test]
// fn test_get_amount_out() {
//     let env = create_test_env();
//     let amount_in = 100;
//     let reserve_in = 1000;
//     let reserve_out = 2000;

//     let amount_out = RaumFiV2Library::get_amount_out(&env, amount_in, reserve_in, reserve_out).unwrap();

//     assert_eq!(amount_out, 180);
// }

// #[test]
// fn test_get_amount_in() {
//     let env = create_test_env();
//     let amount_out = 180;
//     let reserve_in = 1000;
//     let reserve_out = 2000;

//     let amount_in = RaumFiV2Library::get_amount_in(&env, amount_out, reserve_in, reserve_out).unwrap();

//     assert_eq!(amount_in, 101);
// }

// #[test]
// fn test_get_amounts_out() {
//     let env = create_test_env();
//     let factory = Address::generate(&env);
//     let token_a = Address::generate(&env);
//     let token_b = Address::generate(&env);
//     let token_c = Address::generate(&env);

//     // Deploy mock pair contracts
//     let pair_id = env.register_contract(None, mock_pair::MockPair);
//     env.as_contract(&factory, || {
//         env.deployer().with_current_contract(pair_id);
//     });

//     let path = vec![&env, token_a, token_b, token_c];
//     let amount_in = 100;

//     let amounts = RaumFiV2Library::get_amounts_out(&env, factory, amount_in, path).unwrap();

//     assert_eq!(amounts.len(), 3);
//     assert_eq!(amounts.get(0).unwrap(), 100);
//     assert!(amounts.get(1).unwrap() > 0);
//     assert!(amounts.get(2).unwrap() > 0);
// }

// #[test]
// fn test_get_amounts_in() {
//     let env = create_test_env();
//     let factory = Address::generate(&env);
//     let token_a = Address::generate(&env);
//     let token_b = Address::generate(&env);
//     let token_c = Address::generate(&env);

//     // Deploy mock pair contracts
//     let pair_id = env.register_contract(None, mock_pair::MockPair);
//     env.as_contract(&factory, || {
//         env.deployer().with_current_contract(pair_id);
//     });

//     let path = vec![&env, token_a, token_b, token_c];
//     let amount_out = 100;

//     let amounts = RaumFiV2Library::get_amounts_in(&env, factory, amount_out, path).unwrap();

//     assert_eq!(amounts.len(), 3);
//     assert!(amounts.get(0).unwrap() > 0);
//     assert!(amounts.get(1).unwrap() > 0);
//     assert_eq!(amounts.get(2).unwrap(), 100);
// }

// #[test]
// fn test_calculate_k() {
//     let reserve_a = 1000;
//     let reserve_b = 2000;

//     let k = RaumFiV2Library::calculate_k(reserve_a, reserve_b).unwrap();

//     assert_eq!(k, 2_000_000);
// }

// #[test]
// fn test_optimal_liquidity() {
//     let env = create_test_env();
//     let amount_a_desired = 100;
//     let amount_b_desired = 200;
//     let amount_a_min = 90;
//     let amount_b_min = 180;
//     let reserve_a = 1000;
//     let reserve_b = 2000;

//     let (amount_a, amount_b) = optimal_liquidity(
//         &env,
//         amount_a_desired,
//         amount_b_desired,
//         amount_a_min,
//         amount_b_min,
//         reserve_a,
//         reserve_b,
//     )
//     .unwrap();

//     assert_eq!(amount_a, 100);
//     assert_eq!(amount_b, 200);
// }

// #[test]
// fn test_calculate_price_impact() {
//     let env = create_test_env();
//     let amount_in = 100;
//     let amount_out = 180;
//     let reserve_in = 1000;
//     let reserve_out = 2000;

//     let price_impact = calculate_price_impact(&env, amount_in, amount_out, reserve_in, reserve_out).unwrap();

//     assert!(price_impact > 0 && price_impact < 1000); // Price impact should be between 0% and 10%
// }

// #[test]
// fn test_calculate_liquidity_minted() {
//     let env = create_test_env();
//     let total_supply = 1000;
//     let amount_a = 100;
//     let amount_b = 200;
//     let reserve_a = 1000;
//     let reserve_b = 2000;

//     let liquidity = calculate_liquidity_minted(&env, total_supply, amount_a, amount_b, reserve_a, reserve_b).unwrap();

//     assert!(liquidity > 0);
// }

// #[test]
// fn test_calculate_burn_amounts() {
//     let env = create_test_env();
//     let liquidity = 100;
//     let total_supply = 1000;
//     let reserve_a = 1000;
//     let reserve_b = 2000;

//     let (amount_a, amount_b) = calculate_burn_amounts(&env, liquidity, total_supply, reserve_a, reserve_b).unwrap();

//     assert_eq!(amount_a, 100);
//     assert_eq!(amount_b, 200);
// }

// #[test]
// fn test_is_constant_product_maintained() {
//     let reserve_a = 1000;
//     let reserve_b = 2000;
//     let new_reserve_a = 1100;
//     let new_reserve_b = 1818; // Approximately maintains the constant product

//     assert!(is_constant_product_maintained(reserve_a, reserve_b, new_reserve_a, new_reserve_b));

//     let bad_new_reserve_b = 1800; // Does not maintain the constant product
//     assert!(!is_constant_product_maintained(reserve_a, reserve_b, new_reserve_a, bad_new_reserve_b));
// }

// #[test]
// fn test_calculate_next_sqrt_price() {
//     let sqrt_price_x96 = 79228162514264337593543950336_i128; // 1.0 in Q96 format
//     let liquidity = 1000000;
//     let amount = 1000;

//     let next_sqrt_price_zero_for_one = calculate_next_sqrt_price(sqrt_price_x96, liquidity, amount, true);
//     let next_sqrt_price_one_for_zero = calculate_next_sqrt_price(sqrt_price_x96, liquidity, amount, false);

//     assert!(next_sqrt_price_zero_for_one < sqrt_price_x96);
//     assert!(next_sqrt_price_one_for_zero > sqrt_price_x96);
// }
