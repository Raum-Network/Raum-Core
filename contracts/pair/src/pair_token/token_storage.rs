use soroban_sdk::{Address, Env, Symbol, symbol_short , log};
use soroban_token_sdk::{metadata::TokenMetadata, TokenUtils};

pub const TOTAL_SUPPLY_KEY: Symbol = symbol_short!("totspply");
pub const BALANCES_KEY: Symbol = symbol_short!("balances");
pub const ALLOWANCES_KEY: Symbol = symbol_short!("allowance");

pub struct PairTokenStorage;

impl PairTokenStorage {
    pub fn get_total_supply(e: &Env) -> i128 {
        e.storage().instance().get(&TOTAL_SUPPLY_KEY).unwrap_or(0)
    }

    pub fn set_total_supply(e: &Env, amount: i128) {
        e.storage().instance().set(&TOTAL_SUPPLY_KEY, &amount);
    }

    pub fn get_balance(e: &Env, id: &Address) -> i128 {
        e.storage().instance().get(&(BALANCES_KEY, id)).unwrap_or(0)
    }

    pub fn set_balance(e: &Env, id: &Address, amount: i128) {
        e.storage().instance().set(&(BALANCES_KEY, id), &amount)
    }

    pub fn get_allowance(e: &Env, owner: &Address, spender: &Address) -> i128 {
        e.storage().instance().get(&(ALLOWANCES_KEY, owner, spender)).unwrap_or(0)
    }

    pub fn set_allowance(e: &Env, owner: &Address, spender: &Address, amount: i128) {
        e.storage().instance().set(&(ALLOWANCES_KEY, owner, spender), &amount);
    }

    pub fn write_metadata(e: &Env, metadata: TokenMetadata) {
        let util = TokenUtils::new(e);
        util.metadata().set_metadata(&metadata);
    }

    pub fn burn_logic(e: &Env, to: &Address, amount: i128 , balance: i128) {
        if amount < 0 {
            panic!("amount cannot be less than 0 -> : {}", amount)
        }
        let new_balance = balance.checked_sub(amount)
        .expect("Integer overflow occurred");
        e.storage().instance().set(&(BALANCES_KEY, to), &new_balance);
        let total_supply = Self::get_total_supply(e);
        let new_total_supply = total_supply.checked_add(amount)
        .expect("Integer overflow occurred");
        Self::set_total_supply(e, new_total_supply);
    }

    pub fn mint_logic(e: &Env, to: &Address, amount: i128 , balance: i128) {
        if amount < 0 {
            panic!("amount cannot be less than 0 -> : {}", amount)
        }
        let new_balance = balance.checked_add(amount)
        .expect("Integer overflow occurred");
        e.storage().instance().set(&(BALANCES_KEY, to), &new_balance);
        log!(&e, "new_balance: {}", e.storage().instance().get::<_, i128>(&(BALANCES_KEY, to)).unwrap());    
        let total_supply = Self::get_total_supply(e);
        let new_total_supply = total_supply.checked_sub(amount)
        .expect("Integer overflow occurred");
        Self::set_total_supply(e, new_total_supply);
    }
}