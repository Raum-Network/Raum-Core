#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger, LedgerInfo, Events as EventsExt},
    Address, Env, token , BytesN , log
};

use crate::pair::{RaumFiPair, DataKey, RaumFiPairTrait , RaumFiPairClient};
// use factory::RaumFiFactory;
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
use pair::PairClient;

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



use crate::error::RaumFiPairError;

fn setup_test() -> (Env, RaumFiPairClient<'static>, TokenClient<'static>, TokenClient<'static>, Address , TokenAdminClient<'static>, TokenAdminClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RaumFiPair{});
    let client = RaumFiPairClient::new(&env, &contract_id);

    // Create mock addresses for token0, token1, and factory
    let user_address = soroban_sdk::Address::generate(&env);
    let (token0, token0client) = create_token_contract(&env, &user_address);
    let (token1, token1client) = create_token_contract(&env, &user_address);
    let factory = Address::generate(&env);
    (env, client, token0, token1, factory , token0client, token1client)
}

#[test]
fn test_get_reserves() {
    let env = Env::default();
    // env.mock_all_auths();
    
    let user_address = Address::generate(&env);
    let (token0, staked_client) = create_token_contract(&env, &user_address);
    let (token1, steth_client) = create_token_contract(&env, &user_address);
    let factory = Address::generate(&env);

    // Create and initialize the contract
    let contract_id = env.register_contract(None, RaumFiPair{});
    let client = RaumFiPairClient::new(&env, &contract_id);
    
    // Initialize the contract
    client.initialize(&token0.address, &token1.address, &factory);

    // Set initial reserves using the client
    // client.set_reserves(&100, &200, &12345);
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Reserve0 , &100i128) 
    });
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Reserve1 , &200i128) 
    });
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::BlockTimestampLast , &12345u32) 
    });
    // Get reserves using the client
    let (reserve0, reserve1) = client.get_reserves();


    // Assert the values
    assert_eq!(reserve0, 100);
    assert_eq!(reserve1, 200);
}


#[test]
fn test_mint() {
    let (env, client, token0, token1, factory , token0client, token1client) = setup_test();
    let fee_to_setter = Address::generate(&env);
    let pair_wasm = pair_token_wasm(&env);
    let factory_client = create_factory_contract(&env, &fee_to_setter, &pair_wasm);
    // Set reserves and balances
    env.as_contract(&client.address, || {
        env.storage().instance().set(&DataKey::Reserve0, &1000i128);
        env.storage().instance().set(&DataKey::Reserve1, &1000i128);
        env.storage().instance().set(&DataKey::Unlocked, &true);
        env.storage().instance().set(&DataKey::Token0, &token0.address);
        env.storage().instance().set(&DataKey::Token1, &token1.address);
        env.storage().instance().set(&DataKey::Factory, &factory_client.address);
    });

    let user_address = Address::generate(&env);
    token0client.mint(&user_address, &2000i128);
    token1client.mint(&user_address, &4000i128);

    token0.transfer(&user_address, &client.address, &2000i128);
    token1.transfer(&user_address, &client.address, &4000i128);

    
    let result = client.mint(&user_address);
    assert_eq!(result, 732);

    // Verify the event was published
    let logs = env.events().all();
    assert_eq!(logs.len(), 8); // Mint and Sync events expected
}


#[test]
fn test_locked_mint() {
    let (env, client, token0, token1, factory , token0client, token1client) = setup_test();
    let fee_to_setter = Address::generate(&env);
    let pair_wasm = pair_token_wasm(&env);
    let factory_client = create_factory_contract(&env, &fee_to_setter, &pair_wasm);
    // Set reserves and balances

    // Set locked state
    env.as_contract(&client.address, || {
        env.storage().instance().set(&DataKey::Unlocked, &false);
    });

    // Attempt to mint while locked and expect an error
    let result = client.mint(&fee_to_setter);
    log!(&env, "{}", result);
}

#[test]
fn test_swap() {
    let (env, client, token0, token1, factory, token0client, token1client) = setup_test();
    let user = Address::generate(&env);
    let pair_wasm = pair_token_wasm(&env);
    let factory_client = create_factory_contract(&env, &user, &pair_wasm);
    // Initialize contract and set initial state
    client.initialize(&token0.address, &token1.address, &factory_client.address);
   

    // Mint tokens to user and transfer to contract
    token0client.mint(&user, &50_000_000);
    token0.transfer(&user, &client.address, &50_000_000);
    token1client.mint(&user, &100_000_000);
    token1.transfer(&user, &client.address, &100_000_000);
    client.mint(&user);
    token0client.mint(&user, &10_000_000);
    token0.transfer(&user, &client.address, &10_000_000);
    // Perform swap
    let result = client.swap(&0, &16624979, &user);
   

    // Check new reserves
    let (reserve0, reserve1) = client.get_reserves();
    assert_eq!(50_000_000_i128.checked_add(10_000_000).unwrap(), reserve0);
    assert_eq!(100_000_000_i128.checked_sub(16624979).unwrap(), reserve1);
}

#[test]
fn test_burn() {
    let (env, client, token0, token1, factory, token0client, token1client) = setup_test();
    let user = Address::generate(&env);
    let pair_wasm = pair_token_wasm(&env);
    let factory_client = create_factory_contract(&env, &user, &pair_wasm);
    // Initialize contract and set initial state
    client.initialize(&token0.address, &token1.address, &factory_client.address);
    token0client.mint(&user, &3_000_0000);
    token0.transfer(&user, &client.address, &3_000_000);
    token1client.mint(&user, &3_000_0000);
    token1.transfer(&user, &client.address, &2_000_000);
    client.mint(&user);

    // Burn liquidity tokens
    let result = client.burn(&user);
    // Check new reserves (should be lower)
    let (reserve0, reserve1) = client.get_reserves();
    log!(&env, "reserve0: {}", reserve0);
    log!(&env, "reserve1: {}", reserve1);
    assert!(reserve0 < 3_000_0000);
    assert!(reserve1 < 2_000_0000);
}
