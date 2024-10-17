//! This module contains the implementation of the RaumFiV2Library contract.
//! It provides various functions to interact with the RaumFiV2Router contract.
//! The functions are designed to work with the RaumFiV2Pair contract.
//! The functions are designed to work with the RaumFiV2Router contract.


// use num_integer::Roots;
use soroban_sdk::{
     contracttype, Address, Bytes,  Env,  Vec, xdr::ToXdr , log
};

use crate::error::RaumFiLibraryError;
use num_integer::Roots; 


mod pair {
    soroban_sdk::contractimport!(
        file = "D:/RaumFiV2/RaumFiV2/target/wasm32-unknown-unknown/release/pair.wasm"
    );
}
use pair::Client as PairClient;

#[contracttype]
pub enum DataKey {
    Factory,
}

pub trait CheckedCeilingDiv {
    fn checked_ceiling_div(self, divisor: i128) -> Option<i128>;
}

impl CheckedCeilingDiv for i128 {
     fn checked_ceiling_div(self, divisor: i128) -> Option<i128> {
        if divisor == 0 {
            return None;
        }
        
        let quotient = self / divisor;
        let remainder = self % divisor;
        
        if remainder == 0 {
            Some(quotient)
        } else {
            quotient.checked_add(1)
        }
    }
}


    pub fn sort_tokens( token_a: Address, token_b: Address) -> Result<(Address, Address), RaumFiLibraryError> {
        if token_a == token_b {
            return Err(RaumFiLibraryError::IdenticalAddresses);
        }
        let (token0, token1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        Ok((token0, token1))
    }

    
    pub fn pair_for(env: &Env, factory: Address, token_a: Address, token_b: Address) -> Result<Address, RaumFiLibraryError> {
        let (token0, token1) = sort_tokens(token_a, token_b)?;
        let mut salt = Bytes::new(env);

        salt.append(&token0.to_xdr(env));
        salt.append(&token1.to_xdr(env));

        let salt_hash = env.crypto().sha256(&salt);
        let deployer_with_address = env.deployer().with_address(factory, salt_hash);
        
        let pair_address = deployer_with_address.deployed_address();
        Ok(pair_address)
    }

    // Fetch and sort the reserves for a pair
    pub fn get_reserves(
        env: &Env,
        factory: Address,
        token_a: Address,
        token_b: Address,
    ) -> Result<(i128, i128), RaumFiLibraryError> {
        let (token0, token1) = sort_tokens(token_a.clone(), token_b.clone())?;
        let pair = pair_for(env, factory.clone(), token_a.clone(), token_b.clone())?;
        let token_pair = PairClient::new(env, &pair);
        let (reserve0, reserve1) = token_pair.get_reserves();
        Ok(if token_a == token0 {
            (reserve0, reserve1)
        } else {
            (reserve1, reserve0)
        })
    }

    // Calculate equivalent amount of the other asset
    pub fn calculate_quote(env: &Env, amount_a: i128, reserve_a: i128, reserve_b: i128) -> Result<i128, RaumFiLibraryError> {
        if amount_a == 0 {
            return Err(RaumFiLibraryError::InsufficientAmount);
        }
        if reserve_a == 0 || reserve_b == 0 {
            return Err(RaumFiLibraryError::InsufficientLiquidity);
        }
        Ok((amount_a as i128)
            .checked_mul(reserve_b)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_div(reserve_a)
            .ok_or(RaumFiLibraryError::DivisionByZero)?)
    }

    // Calculate the maximum output amount of the other asset
    pub fn calculate_amount_out(env: &Env, amount_in: i128, reserve_in: i128, reserve_out: i128) -> Result<i128, RaumFiLibraryError> {
        if amount_in <= 0 {
            return Err(RaumFiLibraryError::InsufficientAmount);
        }
        if reserve_in <= 0 || reserve_out <= 0 {
            return Err(RaumFiLibraryError::InsufficientLiquidity);
        }
        
        // Calculate fee (0.3% of amount_in)
        let fee = (amount_in * 3).checked_div(1000).ok_or(RaumFiLibraryError::DivisionByZero)?;
        let fee = if fee == 0 { 1 } else { fee };
        
        let amount_in_with_fee = amount_in.checked_sub(fee).ok_or(RaumFiLibraryError::Overflow)?;
        let numerator = amount_in_with_fee.checked_mul(reserve_out).ok_or(RaumFiLibraryError::Overflow)?;
        let denominator = reserve_in.checked_add(amount_in_with_fee).ok_or(RaumFiLibraryError::Overflow)?;
        
        Ok(numerator.checked_div(denominator).ok_or(RaumFiLibraryError::DivisionByZero)?)
    }

    // Calculate the required input amount of the other asset
    pub fn calculate_amount_in(env: &Env, amount_out: i128, reserve_in: i128, reserve_out: i128) -> Result<i128, RaumFiLibraryError> {
        if amount_out <= 0 {
            return Err(RaumFiLibraryError::InsufficientAmount);
        }
        if reserve_in <= 0 || reserve_out <= 0 {
            return Err(RaumFiLibraryError::InsufficientLiquidity);
        }

        let numerator = reserve_in
            .checked_mul(amount_out)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_mul(1000)
            .ok_or(RaumFiLibraryError::Overflow)?;
        let denominator = reserve_out
            .checked_sub(amount_out)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_mul(997)
            .ok_or(RaumFiLibraryError::Overflow)?;
        let division_result = numerator
            .checked_ceiling_div(denominator)
            .ok_or(RaumFiLibraryError::DivisionByZero)?.checked_add(1)
            .ok_or(RaumFiLibraryError::Overflow)?;
        Ok(division_result)
        // log!(&env, "division_result: {}", division_result , "numerator: " , numerator , "denominator: " , denominator);
        // let result = if division_result == 0 { division_result + 1 } else { division_result };
        // result
        //     .checked_add(1)
        //     .ok_or(RaumFiLibraryError::Overflow)
    }

    // Perform chained getAmountOut calculations on any number of pairs
    pub fn get_amounts_out(
        env: &Env,
        factory: Address,
        amount_in: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, RaumFiLibraryError> {
        if path.len() < 2 {
            return Err(RaumFiLibraryError::InvalidPath);
        }
        let mut amounts = Vec::new(env);
        amounts.push_back(amount_in);
        for i in 0..path.len() - 1 {
            let (reserve_in, reserve_out) =
                get_reserves(env, factory.clone(), path.get(i).unwrap(), path.get(i + 1).unwrap())?;
            let amount_out =
                calculate_amount_out(env, amounts.get(i).unwrap(), reserve_in, reserve_out)?;
            amounts.push_back(amount_out);
        }
        Ok(amounts)
    }

    // Perform chained getAmountIn calculations on any number of pairs
    pub fn get_amounts_in(
        env: &Env,
        factory: Address,
        amount_out: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, RaumFiLibraryError> {
        if path.len() < 2 {
            return Err(RaumFiLibraryError::InvalidPath);
        }
        let mut amounts = Vec::new(env);
        amounts.push_front(amount_out);
        for i in (1..path.len()).rev() {
            let (reserve_in, reserve_out) =
                get_reserves(env, factory.clone(), path.get(i - 1).unwrap(), path.get(i).unwrap())?;
            let amount_in =
                calculate_amount_in(env, amounts.get(0).unwrap(), reserve_in, reserve_out)?;
            amounts.push_front(amount_in);
        }
        Ok(amounts)
    }

    // Add this method to the implementation
    pub fn calculate_k(reserve_a: i128, reserve_b: i128) -> Result<i128, RaumFiLibraryError> {
        reserve_a.checked_mul(reserve_b).ok_or(RaumFiLibraryError::Overflow)
    }

    pub fn optimal_liquidity(
        env: &Env,
        amount_a_desired: i128,
        amount_b_desired: i128,
        amount_a_min: i128,
        amount_b_min: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<(i128, i128), RaumFiLibraryError> {
        if reserve_a == 0 && reserve_b == 0 {
            return Ok((amount_a_desired, amount_b_desired));
        }
    
        let amount_b_optimal = calculate_quote(env, amount_a_desired, reserve_a, reserve_b)?;
        if amount_b_optimal <= amount_b_desired {
            if amount_b_optimal < amount_b_min {
                return Err(RaumFiLibraryError::InsufficientBAmount);
            }
            return Ok((amount_a_desired, amount_b_optimal));
        }
    
        let amount_a_optimal = calculate_quote(env, amount_b_desired, reserve_b, reserve_a)?;
        assert!(amount_a_optimal <= amount_a_desired);
        if amount_a_optimal < amount_a_min {
            return Err(RaumFiLibraryError::InsufficientAAmount);
        }
        Ok((amount_a_optimal, amount_b_desired))
    }
    
    // Calculate the price impact of a trade
    pub fn calculate_price_impact(
        env: &Env,
        amount_in: i128,
        amount_out: i128,
        reserve_in: i128,
        reserve_out: i128,
    ) -> Result<i128, RaumFiLibraryError> {
        let mid_price = calculate_quote(env, 10u128.pow(18) as i128, reserve_in, reserve_out)?;
        let execution_price = calculate_quote(env, 10u128.pow(18) as i128, amount_in, amount_out)?;
        let price_impact = mid_price
            .checked_sub(execution_price)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_mul(10000)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_div(mid_price)
            .ok_or(RaumFiLibraryError::DivisionByZero)?;
        Ok(price_impact)
    }
    
    // Calculate the liquidity minted from depositing tokens
    pub fn calculate_liquidity_minted(
        env: &Env,
        total_supply: i128,
        amount_a: i128,
        amount_b: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<i128, RaumFiLibraryError> {
        let mut liquidity: i128;
        if total_supply == 0 {
            liquidity = (amount_a.checked_mul(amount_b).ok_or(RaumFiLibraryError::Overflow)?).sqrt();
            liquidity = liquidity.checked_sub(1000).ok_or(RaumFiLibraryError::Overflow)?; // Minimum liquidity
        } else {
            let liquidity_a = amount_a
                .checked_mul(total_supply)
                .ok_or(RaumFiLibraryError::Overflow)?
                .checked_div(reserve_a)
                .ok_or(RaumFiLibraryError::DivisionByZero)?;
            let liquidity_b = amount_b
                .checked_mul(total_supply)
                .ok_or(RaumFiLibraryError::Overflow)?
                .checked_div(reserve_b)
                .ok_or(RaumFiLibraryError::DivisionByZero)?;
            liquidity = liquidity_a.min(liquidity_b);
        }
        if liquidity <= 0 {
            return Err(RaumFiLibraryError::InsufficientLiquidityMinted);
        }
        Ok(liquidity)
    }
    
    // Calculate the amount of tokens received when burning liquidity
    pub fn calculate_burn_amounts(
        env: &Env,
        liquidity: i128,
        total_supply: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<(i128, i128), RaumFiLibraryError> {
        if liquidity > total_supply {
            return Err(RaumFiLibraryError::InsufficientTotalSupply);
        }
        let amount_a = liquidity
            .checked_mul(reserve_a)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_div(total_supply)
            .ok_or(RaumFiLibraryError::DivisionByZero)?;
        let amount_b = liquidity
            .checked_mul(reserve_b)
            .ok_or(RaumFiLibraryError::Overflow)?
            .checked_div(total_supply)
            .ok_or(RaumFiLibraryError::DivisionByZero)?;
        if amount_a <= 0 || amount_b <= 0 {
            return Err(RaumFiLibraryError::InsufficientLiquidityBurned);
        }
        Ok((amount_a, amount_b))
    }
    
    pub fn is_constant_product_maintained(
        env: &Env,
        reserve_a: i128,
        reserve_b: i128,
        new_reserve_a: i128,
        new_reserve_b: i128,
    ) -> bool {
        if let (Ok(k_before), Ok(k_after)) = (
            calculate_k(reserve_a, reserve_b),
            calculate_k(new_reserve_a, new_reserve_b)
        ) {
            log!(env, "k_before: {}", k_before);
            log!(env, "k_after: {}", k_after);
            k_after >= k_before
        } else {
            false // Return false if there's an error in calculation
        }
    }





