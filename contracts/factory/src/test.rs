#![cfg(test)]

use crate::{FactoryContract, FactoryContractClient,  RaumFiV2Factory, Error};
use soroban_sdk::{testutils::Address as TestAddress, Address, BytesN, Env, symbol_short, IntoVal, Symbol , token , Vec , String};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

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
use pair::{PairClient, WASM};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let fee_to_setter = Address::generate(&env);

    let contract_id = env.register_contract(None, FactoryContract{});
    let client = FactoryContractClient::new(&env, &contract_id);
    let pair_wasm = pair_token_wasm(&env);  
    
    // Initialize the contract
    client.initialize(&fee_to_setter, &pair_wasm);
    let stored_fee_setter:Address = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&symbol_short!("feesetr"))
        .unwrap()
    });

    // panic!("{:?}", stored_fee_setter);
    assert_eq!(stored_fee_setter, fee_to_setter);

    let stored_pairs: BytesN<32> = env.as_contract(&contract_id, || {
        env.storage().instance().get(&Symbol::short("pair_hash"))
        .unwrap()
    });
    assert_eq!(stored_pairs, pair_wasm);

    let stored_pairs: u32 = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&Symbol::short("pairs"))
        .unwrap()
    });
    assert_eq!(stored_pairs, 0);
}

#[test]
fn test_create_pair() {
    let env = Env::default();

    env.mock_all_auths();

    let fee_to_setter = Address::generate(&env);

    let contract_id = env.register_contract(None, FactoryContract{});
    let client = FactoryContractClient::new(&env, &contract_id);
    let pair_wasm = pair_token_wasm(&env);  
    let token_0 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));
    let token_1 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));
    soroban_sdk::contractimport!(file = "../token/target/wasm32-unknown-unknown/release/rntoken.wasm");
    pub type TokenClient<'a> = Client<'a>;
    token_0.initialize(&fee_to_setter, &7, &String::from_str(&env, "Token 0"), &String::from_str(&env, "TOKEN0"));
    token_1.initialize(&fee_to_setter, &7, &String::from_str(&env, "Token 1"), &String::from_str(&env, "TOKEN1"));
    // Initialize the contract
    client.initialize(&fee_to_setter, &pair_wasm);
 
    let pair = client.create_new_pair(&token_0.address, &token_1.address);

    // Verify the pair has been created in storage
    let pairs: u32 = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&symbol_short!("pairs"))
        .unwrap()
    });

    assert_eq!(pairs, 1);
    // assert_eq!(pairs.get(0), Some(&pair).cloned());

    // Check the pair is stored in the pair map
    let stored_pair = client.get_pair( &token_0.address, &token_1.address).unwrap();
    assert_eq!(stored_pair, pair);
    let same_pair = client.get_pair( &token_1.address, &token_0.address).unwrap();
    assert_eq!(stored_pair, same_pair);
}

#[test]
#[should_panic(expected = "1")]
fn test_create_pair_identical_addresses() {
    let env = Env::default();

    env.mock_all_auths();

    let fee_to_setter = Address::generate(&env);

    let contract_id = env.register_contract(None, FactoryContract{});
    let client = FactoryContractClient::new(&env, &contract_id);
    let pair_wasm = pair_token_wasm(&env);  
    let token_0 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));
    let token_1 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));

    client.initialize(&fee_to_setter, &pair_wasm);

    // This should panic with IdenticalAddresses error
    RaumFiV2Factory::create_pair(&env, &token_0.address, &token_0.address);
}

#[test]
#[should_panic(expected = "3")]
fn test_create_pair_exists() {
    let env = Env::default();

    env.mock_all_auths();

    let fee_to_setter = Address::generate(&env);

    let contract_id = env.register_contract(None, FactoryContract{});
    let client = FactoryContractClient::new(&env, &contract_id);
    let pair_wasm = pair_token_wasm(&env);  
    let token_0 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));
    let token_1 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));

    client.initialize(&fee_to_setter, &pair_wasm);

    // This should panic with IdenticalAddresses error
    client.create_new_pair( &token_0.address, &token_1.address);
    client.create_new_pair( &token_0.address, &token_1.address);
}

#[test]
fn test_set_fee_to() {
    let env = Env::default();

    env.mock_all_auths();

    let fee_to_setter = Address::generate(&env);

    let contract_id = env.register_contract(None, FactoryContract{});
    let client = FactoryContractClient::new(&env, &contract_id);
    let pair_wasm = pair_token_wasm(&env);  
    let token_0 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));
    let token_1 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));

    client.initialize(&fee_to_setter, &pair_wasm);

    // Authorized fee setter sets a new fee recipient
    // env.set_auths(fee_to_setter.clone());
    client.set_fee_to( &fee_to_setter.clone());

    let stored_fee_to: Address = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&Symbol::short("feeto"))
        .unwrap()
    });
    assert_eq!(stored_fee_to, fee_to_setter);
}

#[test]
#[should_panic(expected = "4")]
fn test_set_fee_to_unauthorized() {
    let env = Env::default();

    // env.mock_all_auths();

    let fee_to_setter = Address::generate(&env);

    let contract_id = env.register_contract(None, FactoryContract{});
    let client = FactoryContractClient::new(&env, &contract_id);
    let pair_wasm = pair_token_wasm(&env);  
    let token_0 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));
    let token_1 = TokenClient::new(&env, &env.register_contract_wasm(None,WASM));

    client.initialize(&fee_to_setter, &pair_wasm);

    // Unauthorized address tries to set fee recipient, should panic with Forbidden error
    client.set_fee_to(&Address::generate(&env));
}
