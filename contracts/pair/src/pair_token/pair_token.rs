use soroban_sdk::{
    contract, contractimpl, Address, Env, String
};
use soroban_token_sdk::TokenUtils;

use crate::pair_token::{PairTokenEvents, PairTokenStorage, PairTokenError};


#[contract]
pub struct PairToken;


#[contractimpl]
impl PairToken {

    pub fn mint_token(env: &Env, to: Address, amount: i128) -> Result<(), PairTokenError> {
        if amount <= 0 {
            return Err(PairTokenError::InvalidAmount);
        }
        let balance = Self::balance(env.clone(), to.clone());
        PairTokenStorage::mint_logic(env, &to, amount, balance);
        Ok(())
    }
    
    pub fn burn_token(env: &Env, from: Address, amount: i128) -> Result<(), PairTokenError> {
        if amount < 0 {
            panic!("amount cannot be less than 0 -> : {}", amount)
        }
        let balance = Self::balance(env.clone(), from.clone());
        PairTokenStorage::burn_logic(env, &from, amount, balance);
        Ok(())
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        PairTokenStorage::get_balance(&e, &id)
    }

    pub fn allowance(e: Env, owner: Address, spender: Address) -> i128 {
        PairTokenStorage::get_allowance(&e, &owner, &spender)
    }

    pub fn approve(e: Env, owner: Address, spender: Address, amount: i128) -> bool {
        owner.require_auth();
        PairTokenStorage::set_allowance(&e, &owner, &spender, amount);
        PairTokenEvents::approval(&e, &owner, &spender, &amount);
        true
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) -> bool {
        from.require_auth();
        Self::do_transfer(&e, &from, &to, amount)
    }

    fn do_transfer(e: &Env, from: &Address, to: &Address, amount: i128) -> bool {
        let from_balance = Self::balance(e.clone(), from.clone());
        if from_balance < amount {
            panic!("from_balance cannot be less than amount -> : {}", amount);
        }
        PairTokenStorage::set_balance(e, from, from_balance - amount);
        let to_balance = Self::balance(e.clone(), to.clone());
        PairTokenStorage::set_balance(e, to, to_balance + amount);
        PairTokenEvents::transfer(e, from, to, &amount);
        true
    }

    pub fn read_decimal(e: &Env) -> u32 {
        let util = TokenUtils::new(e);
        util.metadata().get_metadata().decimal
    }
    
    pub fn read_name(e: &Env) -> String {
        let util = TokenUtils::new(e);
        util.metadata().get_metadata().name
    }
    
    pub fn read_symbol(e: &Env) -> String {
        let util = TokenUtils::new(e);
        util.metadata().get_metadata().symbol
    }

}
