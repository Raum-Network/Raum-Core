#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{log, symbol_short, token, vec, Address, BytesN, Env , Bytes};
use crate::library::{RaumFiV2LibraryClient  , RaumFiV2Library};


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

    pub fn register_pair_contract(env: &soroban_sdk::Env) -> soroban_sdk::Address {
        let contract_id = env.register_contract_wasm(None, WASM);
        contract_id
    }
}
use pair::{PairClient, DataKey};

fn create_pair_contract<'a>(e: &Env, factory: &Address, token_a: &Address, token_b: &Address) -> PairClient<'a> {
    let mut salt = Bytes::new(e);
    salt.append(&token_a.to_xdr(e));
    salt.append(&token_b.to_xdr(e));

    let salt_hash = e.crypto().sha256(&salt);
    let pair_address = e.deployer().with_address(factory.clone(), salt_hash).deployed_address();
    let pair = PairClient::new(e, &pair_address);
    pair.initialize(token_a, token_b, factory);
    pair
}

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
fn test_quote() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token_a , token_a_client) = create_token_contract(&env, &admin);
    let (token_b , token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);

    let amount_a: i128 = 100;
    let reserve_a: i128 = 1000;
    let reserve_b: i128 = 2000;

    let amount_b =client.quote(&amount_a, &reserve_a, &reserve_b);

    assert_eq!(amount_b, 200);
}

#[test]
fn test_get_amount_out() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token_a , token_a_client) = create_token_contract(&env, &admin);
    let (token_b , token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);

    let amount_in: i128 = 3;
    let reserve_in: i128 = 100;
    let reserve_out: i128 = 100;

    let amount_out = client.get_amount_out(&amount_in, &reserve_in, &reserve_out);

    assert_eq!(amount_out, 1);
}

#[test]
fn test_get_amount_in() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token_a , token_a_client) = create_token_contract(&env, &admin);
    let (token_b , token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    let amount_out: i128 = 1;
    let reserve_in: i128 = 100;
    let reserve_out: i128 = 100;
    let amount_in = client.get_amount_in(&amount_out, &reserve_in, &reserve_out);

    assert_eq!(amount_in, 3);
}

//Work after router is created
// #[test]
// fn test_get_amounts_out() {
//     let env = Env::default();
//     env.mock_all_auths();
//     let admin = Address::generate(&env);
//     let (token_a, token_a_client) = create_token_contract(&env, &admin);
//     let (token_b, token_b_client) = create_token_contract(&env, &admin);
//     let client = create_test_contract(&env);
//     let pair_wasm = pair_token_wasm(&env);

//     let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
//     // Create the pair contract using the existing function
//     let pair_address = pair::register_pair_contract(&env);
//     let pair_client = pair::PairClient::new(&env, &pair_address);
//     pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);

//     // Initialize the pair contract with some liquidity
//     let initial_liquidity = 10_000i128;
//     env.as_contract(&pair_address, || {
//         env.storage().instance().set(&DataKey::Reserve0, &10_000i128);
//         env.storage().instance().set(&DataKey::Reserve1, &10_000i128);
//     });
    
//     let (reserve0, reserve1) = pair_client.get_reserves();
//     assert_eq!(reserve0, initial_liquidity);
//     assert_eq!(reserve1, initial_liquidity);

//     let path = vec![&env, token_a_client.address.clone(), token_b_client.address.clone()];
//     let amount_in = 100i128;
//     log!(&env, "pair_client.address: {}", pair_client.address, token_a_client.address, token_b_client.address, factory_client.address, client.address);
//     let amounts = client.get_amounts_out(&factory_client.address, &amount_in, &path);

//     assert_eq!(amounts.len(), 2);
//     assert_eq!(amounts.get(0).unwrap(), amount_in);
//     assert!(amounts.get(1).unwrap() > 0);
// }

//Work after router is created
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

#[test]
fn test_calculate_k() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (token_a, token_a_client) = create_token_contract(&env, &admin);
    let (token_b, token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
    // Create the pair contract using the existing function
    let pair_address = pair::register_pair_contract(&env);
    let pair_client = pair::PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);

    let reserve_a:i128 = 1000;
    let reserve_b:i128 = 2000;

    let k = client.calculate_k(&reserve_a, &reserve_b);

    assert_eq!(k, 2_000_000);
}

#[test]
fn test_optimal_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (token_a, token_a_client) = create_token_contract(&env, &admin);
    let (token_b, token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
    // Create the pair contract using the existing function
    let pair_address = pair::register_pair_contract(&env);
    let pair_client = pair::PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);
    let amount_a_desired = 100;
    let amount_b_desired = 200;
    let amount_a_min = 90;
    let amount_b_min = 180;
    let reserve_a = 1000;
    let reserve_b = 2000;

    let (amount_a, amount_b) = client.optimal_liquidity(
        &amount_a_desired,
        &amount_b_desired,
        &amount_a_min,
        &amount_b_min,
        &reserve_a,
        &reserve_b,
    );

    assert_eq!(amount_a, 100);
    assert_eq!(amount_b, 200);
}

#[test]
fn test_calculate_price_impact() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (token_a, token_a_client) = create_token_contract(&env, &admin);
    let (token_b, token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
    // Create the pair contract using the existing function
    let pair_address = pair::register_pair_contract(&env);
    let pair_client = pair::PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);
    let amount_in = 100;
    let amount_out = 189;
    let reserve_in = 10000;
    let reserve_out:i128 = 20000;

    let price_impact = client.calculate_price_impact(&amount_in, &amount_out, &reserve_in, &reserve_out);
    log!(&env, "price_impact: {}", price_impact);
    assert!(price_impact > 0 && price_impact < 1000); // Price impact should be between 0% and 10%
}

#[test]
fn test_calculate_liquidity_minted() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (token_a, token_a_client) = create_token_contract(&env, &admin);
    let (token_b, token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
    // Create the pair contract using the existing function
    let pair_address = pair::register_pair_contract(&env);
    let pair_client = pair::PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);
    let total_supply = 1000;
    let amount_a = 100;
    let amount_b = 200;
    let reserve_a = 1000;
    let reserve_b = 2000;

    let liquidity = client.calculate_liquidity_minted(&total_supply, &amount_a, &amount_b, &reserve_a, &reserve_b);
    log!(&env, "liquidity: {}", liquidity);
    assert!(liquidity > 0);
}

#[test]
fn test_calculate_burn_amounts() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (token_a, token_a_client) = create_token_contract(&env, &admin);
    let (token_b, token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
    // Create the pair contract using the existing function
    let pair_address = pair::register_pair_contract(&env);
    let pair_client = pair::PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);
    let liquidity = 100;
    let total_supply = 1000;
    let reserve_a = 1000;
    let reserve_b = 2000;

    let (amount_a, amount_b) = client.calculate_burn_amounts(&liquidity, &total_supply, &reserve_a, &reserve_b);
    log!(&env, "amount_a: {}", amount_a);
    log!(&env, "amount_b: {}", amount_b);
    assert_eq!(amount_a, 100);
    assert_eq!(amount_b, 200);
}

#[test]
fn test_is_constant_product_maintained() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (token_a, token_a_client) = create_token_contract(&env, &admin);
    let (token_b, token_b_client) = create_token_contract(&env, &admin);
    let client = create_test_contract(&env);
    let pair_wasm = pair_token_wasm(&env);

    let factory_client = create_factory_contract(&env, &admin, &pair_wasm);
    
    // Create the pair contract using the existing function
    let pair_address = pair::register_pair_contract(&env);
    let pair_client = pair::PairClient::new(&env, &pair_address);
    pair_client.initialize(&token_a_client.address, &token_b_client.address, &factory_client.address);
    let reserve_a = 1000;
    let reserve_b = 2000;
    let new_reserve_a = 1100;
    let new_reserve_b = 1900; // Approximately maintains the constant product
    let constant_product_maintained = client.is_constant_product_maintained(&reserve_a, &reserve_b, &new_reserve_a, &new_reserve_b);
    log!(&env, "constant_product_maintained: {}", constant_product_maintained);
    // assert!(client.is_constant_product_maintained(&reserve_a, &reserve_b, &new_reserve_a, &new_reserve_b));
    assert_eq!(constant_product_maintained, true);
    let bad_new_reserve_b = 1800; // Does not maintain the constant product
    let constant_product_maintained = client.is_constant_product_maintained(&reserve_a, &reserve_b, &new_reserve_a, &bad_new_reserve_b);
    log!(&env, "constant_product_maintained: {}", constant_product_maintained);
    assert_eq!(constant_product_maintained, false);
}


