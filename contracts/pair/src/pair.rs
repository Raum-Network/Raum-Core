#![no_std]

use core::panic;

use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, log, token, Address, Env, Symbol, Vec
};

use crate::pair_token::{PairToken, PairTokenStorage};
use num_integer::Roots; 
use crate::error::RaumFiPairError;

use crate::interface::RaumFiFactoryClient;

const MINIMUM_LIQUIDITY: i128 = 1000;

#[contracttype]
#[derive(Clone)]
 pub enum DataKey {
    Factory,
    Token0,
    Token1,
    Reserve0,
    Reserve1,
    BlockTimestampLast,
    Price0CumulativeLast,
    Price1CumulativeLast,
    KLast,
    Unlocked,
}

 enum IdenticalPairError {
    /// RaumFiFactory: token_a and token_b have identical addresses
    CreatePairIdenticalTokens = 901,
    PairTokenError = 902,
    InvalidAmount = 903,
}

impl From<IdenticalPairError> for RaumFiPairError {
    fn from(pair_error: IdenticalPairError) -> Self {
        match pair_error {
            IdenticalPairError::CreatePairIdenticalTokens => RaumFiPairError::CreatePairIdenticalTokens,
            IdenticalPairError::PairTokenError => RaumFiPairError::PairTokenError,
            IdenticalPairError::InvalidAmount => RaumFiPairError::InvalidAmount,
            
        }
    }
}

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "RaumFi V2 DEX"
);


 pub trait RaumFiPairTrait{
    fn initialize(env: Env, token0: Address, token1: Address, factory: Address) -> Result<(), RaumFiPairError>;
    fn get_reserves(env: Env) -> (i128, i128);
    fn mint(env: Env, to: Address) -> Result<i128, RaumFiPairError >;
    fn mint_fee(env: Env, reserve0: i128, reserve1: i128) -> Result<bool, RaumFiPairError>;
    fn get_balance(env: Env, token_key: DataKey) -> i128;
    fn check_locked(env: &Env) -> Result<(), RaumFiPairError>;

    // New functions for swap and burn
    fn swap(env: Env, amount0_out: i128, amount1_out: i128, to: Address) -> Result<(), RaumFiPairError>;
    fn burn(env: Env, to: Address) -> Result<(i128, i128), RaumFiPairError>;
}

#[contract]
pub struct RaumFiPair;

#[contractimpl]
impl RaumFiPairTrait for RaumFiPair {
    fn initialize(env: Env, token0: Address, token1: Address, factory: Address) -> Result<(), RaumFiPairError> {
        if env.storage().instance().has(&DataKey::Factory) {
            return Err(RaumFiPairError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Factory, &factory);
        env.storage().instance().set(&DataKey::Token0, &token0);
        env.storage().instance().set(&DataKey::Token1, &token1);
        env.storage().instance().set(&DataKey::Unlocked, &true);

        Ok(())
    }

     fn get_reserves(env: Env) -> (i128, i128) {
        let reserve0: i128 = env.storage().instance().get(&DataKey::Reserve0).unwrap_or(0);
        let reserve1: i128 = env.storage().instance().get(&DataKey::Reserve1).unwrap_or(0);
        
        (reserve0, reserve1)
    }

     fn mint(env: Env, to: Address) -> Result<i128, RaumFiPairError > {
        Self::check_locked(&env)?;
        let _guard = Guard::new(&env);
        let (reserve0, reserve1) = Self::get_reserves(env.clone());
        
        let balance0 = Self::get_balance(env.clone(), DataKey::Token0);
        let balance1 = Self::get_balance(env.clone(), DataKey::Token1);

        let amount0 = balance0.checked_sub(reserve0).ok_or(IdenticalPairError::InvalidAmount)?;
        let amount1 = balance1.checked_sub(reserve1).ok_or(IdenticalPairError::InvalidAmount)?;

        if amount0 <= 0 {
            return Err(RaumFiPairError::InvalidAmount);
        }

        if amount1 <= 0 {
            return Err(RaumFiPairError::InvalidAmount);
        }

        let fee_on = Self::mint_fee(env.clone(), reserve0, reserve1)?;
        let total_supply = PairTokenStorage::get_total_supply(&env);
        
        let liquidity = if total_supply == 0 {
            let initial_liquidity =(amount0.checked_mul(amount1).unwrap()).sqrt() - MINIMUM_LIQUIDITY;
            match PairToken::mint_token(&env, env.current_contract_address().clone(), MINIMUM_LIQUIDITY) {
                Ok(_) => {},
                Err(_e) => return Err(RaumFiPairError::PairTokenError),
            }
            
            initial_liquidity
        } else {
            let liquidity0 = (amount0.checked_mul(total_supply).unwrap()).checked_div(reserve0).unwrap();
            let liquidity1 = (amount1.checked_mul(total_supply).unwrap()).checked_div(reserve1).unwrap();
            liquidity0.min(liquidity1)
        };
        

        if liquidity <= 0 {
            return Err(RaumFiPairError::InsufficientLiquidityMinted);
        }
        
        
        match PairToken::mint_token(&env, to.clone(), liquidity) {
            Ok(_) => {},
            Err(_e) => return Err(RaumFiPairError::PairTokenError),
        }
        // PairToken::mint(&env, to.clone(), liquidity)?;

        log!(&env, "liquidity: {}", liquidity);
       
        update(&env, balance0, balance1)?;

        if fee_on {
            let k_last = reserve0 as i128 * reserve1 as i128;
            env.storage().instance().set(&DataKey::KLast, &k_last);
        }

        env.events().publish(
            (Symbol::new(&env, "RaumFiPair"), Symbol::new(&env, "Mint")),
            (to, amount0, amount1)
        );

        Ok(liquidity)

    }


    fn mint_fee(env: Env, reserve0: i128, reserve1: i128) -> Result<bool, RaumFiPairError> {
        let factory: Address = env.storage().instance().get(&DataKey::Factory).unwrap();
        let factory_client = RaumFiFactoryClient::new(&env, &factory);
        let fee_on = true;
        let k_last: i128 = env.storage().instance().get(&DataKey::KLast).unwrap_or(0);
        let fee_to = factory_client.get_fee_to();   

        if fee_on {
            if k_last != 0 {
                let root_k = ((reserve0.checked_mul(reserve1).unwrap()).sqrt()) as i128;
                let root_k_last = k_last.sqrt() as i128;
                if root_k > root_k_last {
                    let total_supply = PairTokenStorage::get_total_supply(&env);
                    let numerator = (root_k.saturating_sub(root_k_last)).saturating_mul(total_supply);
                    let denominator = root_k.saturating_mul(5).saturating_add(root_k_last);
                    let liquidity = numerator.saturating_div(denominator);
                    if liquidity != 0 {
                        match PairToken::mint_token(&env, fee_to.clone(), liquidity) {
                            Ok(_) => {},
                            Err(_e) => return Err(RaumFiPairError::PairTokenError),
                        }
                    }
                }
            }
        } else if !k_last == 0 {
            env.storage().instance().set(&DataKey::KLast,  &0i128);
        }

        Ok(fee_on)
    }


    fn get_balance(env: Env, token_key: DataKey) -> i128 {
        let token: Address = env.storage().instance().get(&token_key).unwrap();
        let token_client = token::Client::new(&env, &token);
        token_client.balance(&env.current_contract_address())
    }

    fn check_locked(env: &Env) -> Result<(), RaumFiPairError> {
        let unlocked: bool = env.storage().instance().get(&DataKey::Unlocked).unwrap_or(true);
        if !unlocked {
            return Err(RaumFiPairError::Locked);
        }
        Ok(())
    }

    fn swap(env: Env, amount0_out: i128, amount1_out: i128, to: Address) -> Result<(), RaumFiPairError> {
        Self::check_locked(&env)?;
        let _guard = Guard::new(&env);

        if amount0_out == 0 && amount1_out == 0 {
            return Err(RaumFiPairError::InsufficientOutputAmount);
        }

        let (reserve0, reserve1) = Self::get_reserves(env.clone());

        if amount0_out > reserve0 || amount1_out > reserve1 {
            return Err(RaumFiPairError::InsufficientLiquidity);
        }

        let token0: Address = env.storage().instance().get(&DataKey::Token0).unwrap();
        let token1: Address = env.storage().instance().get(&DataKey::Token1).unwrap();

        if to == token0 || to == token1 {
            return Err(RaumFiPairError::InvalidTo);
        }

        if amount0_out > 0 {
            token::Client::new(&env, &token0).transfer(&env.current_contract_address(), &to, &amount0_out);
        }
        if amount1_out > 0 {
            token::Client::new(&env, &token1).transfer(&env.current_contract_address(), &to, &amount1_out);
        }

        let balance0 = Self::get_balance(env.clone(), DataKey::Token0);
        let balance1 = Self::get_balance(env.clone(), DataKey::Token1);

        let amount0_in = if balance0 > reserve0 - amount0_out { balance0 - (reserve0 - amount0_out) } else { 0 };
        let amount1_in = if balance1 > reserve1 - amount1_out { balance1 - (reserve1 - amount1_out) } else { 0 };

        if amount0_in == 0 && amount1_in == 0 {
            return Err(RaumFiPairError::InsufficientInputAmount);
        }

        let balance0_adjusted = balance0.checked_mul(1000).unwrap() - amount0_in.checked_mul(3).unwrap();
        let balance1_adjusted = balance1.checked_mul(1000).unwrap() - amount1_in.checked_mul(3).unwrap();

        if balance0_adjusted.checked_mul(balance1_adjusted).unwrap() < reserve0.checked_mul(reserve1).unwrap().checked_mul(1000000).unwrap() {
            return Err(RaumFiPairError::K);
        }

        update(&env, balance0, balance1)?;

        env.events().publish(
            (Symbol::new(&env, "RaumFiPair"), Symbol::new(&env, "Swap")),
            (amount0_in, amount1_in, amount0_out, amount1_out, to)
        );

        Ok(())
    }

    fn burn(env: Env, to: Address) -> Result<(i128, i128), RaumFiPairError> {
        Self::check_locked(&env)?;
        let _guard = Guard::new(&env);

        let token0: Address = env.storage().instance().get(&DataKey::Token0).unwrap();
        let token1: Address = env.storage().instance().get(&DataKey::Token1).unwrap();
        let (reserve0, reserve1) = Self::get_reserves(env.clone());

        let balance0 = Self::get_balance(env.clone(), DataKey::Token0);
        let balance1 = Self::get_balance(env.clone(), DataKey::Token1);
        let liquidity = PairTokenStorage::get_balance(&env, &env.current_contract_address());

        let userbalance = liquidity.checked_sub(MINIMUM_LIQUIDITY).unwrap();
        let total_supply = PairTokenStorage::get_total_supply(&env);
        let amount0 = balance0.checked_mul(userbalance).unwrap().checked_div(total_supply).unwrap();
        let amount1 = balance1.checked_mul(userbalance).unwrap().checked_div(total_supply).unwrap();

        if amount0 == 0 || amount1 == 0 {
            return Err(RaumFiPairError::InsufficientLiquidityBurned);
        }

        match PairToken::burn_token(&env, env.current_contract_address(), liquidity) {
            Ok(_) => {},
            Err(_) => return Err(RaumFiPairError::PairTokenError),
        }

        token::Client::new(&env, &token0).transfer(&env.current_contract_address(), &to, &amount0);
        token::Client::new(&env, &token1).transfer(&env.current_contract_address(), &to, &amount1);

        let balance0 = Self::get_balance(env.clone(), DataKey::Token0);
        let balance1 = Self::get_balance(env.clone(), DataKey::Token1);

        update(&env, balance0, balance1)?;

        env.events().publish(
            (Symbol::new(&env, "RaumFiPair"), Symbol::new(&env, "Burn")),
            (to, amount0, amount1)
        );

        Ok((amount0, amount1))
    }
}

fn update(env: &Env, balance0: i128, balance1: i128) -> Result<(), RaumFiPairError> {


    // Update reserves and block timestamp
    env.storage().instance().set(&DataKey::Reserve0, &balance0);
    env.storage().instance().set(&DataKey::Reserve1, &balance1);

    env.events().publish(
        (Symbol::new(&env, "RaumFiPair"), Symbol::new(&env, "Sync")),
        (balance0, balance1)
    );

    Ok(())
}

struct Guard<'a> {
    env: &'a Env,
}

impl<'a> Guard<'a> {
    fn new(env: &'a Env) -> Self {
        env.storage().instance().set(&DataKey::Unlocked, &false);
        Self { env }
    }
}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        self.env.storage().instance().set(&DataKey::Unlocked, &true);
    }
}



