#![cfg(test)]

use super::*;
use router::RaumFiRouterClient;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{log, symbol_short, token, vec, Address, BytesN, Env , Bytes};
use router::RaumFiRouter;


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
use pair::PairClient;

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

fn create_test_contract(env: &Env) -> RaumFiRouterClient<'static> {
    let contract_id = env.register_contract(None, RaumFiRouter{});
    let client = RaumFiRouterClient::new(env, &contract_id);
    client
    
}

#[test]
fn test_quote() {
    let env = Env::default();
    let router = create_test_contract(&env);

    let amount_a = 1000;
    let reserve_a = 10000;
    let reserve_b = 5000;

    let result = router.quote(&amount_a, &reserve_a, &reserve_b);
    assert_eq!(result, 500); // Expected result based on the formula: (amount_a * reserve_b) / reserve_a
}

#[test]
fn test_add_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pair_wasm_hash = pair_token_wasm(&env);
    let factory = create_factory_contract(&env, &admin, &pair_wasm_hash);
    
    let (token_a, token_a_client)  = create_token_contract(&env, &admin);
    let (token_b, token_b_client)  = create_token_contract(&env, &admin);
    let (token_c, token_c_client)  = create_token_contract(&env, &admin);
    token_c_client.mint(&admin, &1000000000000000);
    token_a_client.mint(&admin, &1000000000000000);
    token_b_client.mint(&admin, &1000000000000000);
    
    let router = create_test_contract(&env);
    router.initialize(&factory.address, &token_c_client.address);

    let amount_a_desired = 10000;
    let amount_b_desired = 5000;
    let amount_a_min = 9000;
    let amount_b_min = 4500;
    let deadline = 1000000;
    log!(&env, "token_a_client.address: {}", token_a_client.address , token_b_client.address , factory.address , router.address );
    let (amount_a, amount_b, liquidity) = router.add_liquidity(
        &token_a_client.address,
        &token_b_client.address,
        &amount_a_desired,
        &amount_b_desired,
        &amount_a_min,
        &amount_b_min,
        &admin,
        &deadline,
    );

    let pair = PairClient::new(&env, &factory.get_pair(&token_a_client.address, &token_b_client.address).unwrap());
    assert!(amount_a >= amount_a_min);
    assert!(amount_b >= amount_b_min);
    assert!(liquidity > 0);
}

#[test]
fn test_remove_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pair_wasm_hash = pair_token_wasm(&env);
    let factory = create_factory_contract(&env, &admin, &pair_wasm_hash);
    
    let (token_a, token_a_client)  = create_token_contract(&env, &admin);
    let (token_b, token_b_client)  = create_token_contract(&env, &admin);
    let (token_c, token_c_client)  = create_token_contract(&env, &admin);
    token_c_client.mint(&admin, &1000000000000000);
    token_a_client.mint(&admin, &1000000000000000);
    token_b_client.mint(&admin, &1000000000000000);

    
    
    let router = create_test_contract(&env);
    router.initialize(&factory.address, &token_c_client.address);
    // First, add some liquidity
    

    let (deposit_a, deposit_b, liquidity) = router.add_liquidity(
        &token_a_client.address,
        &token_b_client.address,
        &10000,
        &5000,
        &9000,
        &4500,
        &admin,
        &1000000,
    );

    let pair_client = PairClient::new(&env, &factory.get_pair(&token_a_client.address, &token_b_client.address).unwrap());
    
    // // Now remove the liquidity
    let amount_a_min = 800;
    let amount_b_min = 400;
    let deadline = 2000000;
    log!(&env, "liquidity: {}", liquidity , &admin);
    
    let (amount_a, amount_b) = router.remove_liquidity(
        &token_a_client.address,
        &token_b_client.address,
        &liquidity,
        &amount_a_min,
        &amount_b_min,
        &admin,
        &deadline,
    );
    log!(&env, "liquidity: {}", pair_client.balance(&admin));
    log!(&env, "amount_a: {}", amount_a);
    log!(&env, "amount_b: {}", amount_b);

    assert!(amount_a >= amount_a_min);
    assert!(amount_b >= amount_b_min);
}

// #[test]
// fn test_swap_exact_tokens_for_tokens() {
//     let env = Env::default();
//     let admin = Address::random(&env);
    
//     let token_in = create_token_contract(&env, &admin);
//     let token_out = create_token_contract(&env, &admin);
    
//     let router = create_router(&env);
//     let client = crate::router::Client::new(&env, &router);

//     let amount_in = 100;
//     let amount_out_min = 80;
//     let path = vec![&env, token_in.clone(), token_out.clone()];
//     let to = Address::random(&env);
//     let deadline = 1000000;

//     let amounts = client.swap_exact_tokens_for_tokens(
//         &amount_in,
//         &amount_out_min,
//         &path,
//         &to,
//         &deadline,
//     );

//     assert_eq!(amounts.len(), 2);
//     assert_eq!(amounts.get(0).unwrap(), amount_in);
//     assert!(amounts.get(1).unwrap() >= amount_out_min);
// }

// #[test]
// fn test_swap_tokens_for_exact_tokens() {
//     let env = Env::default();
//     let admin = Address::random(&env);
    
//     let token_in = create_token_contract(&env, &admin);
//     let token_out = create_token_contract(&env, &admin);
    
//     let router = create_router(&env);
//     let client = crate::router::Client::new(&env, &router);

//     let amount_out = 80;
//     let amount_in_max = 110;
//     let path = vec![&env, token_in.clone(), token_out.clone()];
//     let to = Address::random(&env);
//     let deadline = 1000000;

//     let amounts = client.swap_tokens_for_exact_tokens(
//         &amount_out,
//         &amount_in_max,
//         &path,
//         &to,
//         &deadline,
//     );

//     assert_eq!(amounts.len(), 2);
//     assert!(amounts.get(0).unwrap() <= amount_in_max);
//     assert_eq!(amounts.get(1).unwrap(), amount_out);
// }

// #[test]
// fn test_get_amounts_out() {
//     let env = Env::default();
//     let admin = Address::random(&env);
    
//     let token_a = create_token_contract(&env, &admin);
//     let token_b = create_token_contract(&env, &admin);
    
//     let router = create_router(&env);
//     let client = crate::router::Client::new(&env, &router);

//     let amount_in = 100;
//     let path = vec![&env, token_a.clone(), token_b.clone()];

//     let amounts = client.get_amounts_out(&amount_in, &path);

//     assert_eq!(amounts.len(), 2);
//     assert_eq!(amounts.get(0).unwrap(), amount_in);
//     assert!(amounts.get(1).unwrap() > 0);
// }

// #[test]
// fn test_get_amounts_in() {
//     let env = Env::default();
//     let admin = Address::random(&env);
    
//     let token_a = create_token_contract(&env, &admin);
//     let token_b = create_token_contract(&env, &admin);
    
//     let router = create_router(&env);
//     let client = crate::router::Client::new(&env, &router);

//     let amount_out = 80;
//     let path = vec![&env, token_a.clone(), token_b.clone()];

//     let amounts = client.get_amounts_in(&amount_out, &path);

//     assert_eq!(amounts.len(), 2);
//     assert!(amounts.get(0).unwrap() > 0);
//     assert_eq!(amounts.get(1).unwrap(), amount_out);
// }

