use soroban_sdk::{Address, Env, xdr::ToXdr, BytesN, Bytes};
use crate::error::RaumFiRouterError;


/// Generates a unique cryptographic salt value for a pair of token addresses.
///
/// # Returns
///
/// Returns a `BytesN<32>` representing the salt for the given token pair.
fn pair_salt(e: &Env, token_a: Address, token_b: Address) -> BytesN<32> {
    let mut salt = Bytes::new(e);

    // Append the bytes of token_a and token_b to the salt
    salt.append(&token_a.clone().to_xdr(e)); // can be simplified to salt.append(&self.clone().to_xdr(e)); but changes the hash
    salt.append(&token_b.clone().to_xdr(e));

    // Hash the salt using SHA256 to generate a new BytesN<32> value
    e.crypto().sha256(&salt).into()
}

/// Sorts two token addresses in a consistent order.
///
/// # Returns
///
/// Returns `Result<(Address, Address), RaumFiLibraryError>` where `Ok` contains a tuple with the sorted token addresses, and `Err` indicates an error such as identical tokens.
pub fn sort_tokens(token_a: Address, token_b: Address) -> Result<(Address, Address), RaumFiRouterError> {
    if token_a == token_b {
        return Err(RaumFiRouterError::SortIdenticalTokens);
    }

    if token_a < token_b {
        Ok((token_a, token_b))
    } else {
        Ok((token_b, token_a))
    }
}

/// Calculates the deterministic address for a pair without making any external calls.
/// check <https://github.com/paltalabs/deterministic-address-soroban>
/// 
/// # Returns
///
/// Returns `Result<Address, RaumFiLibraryError>` where `Ok` contains the deterministic address for the pair, and `Err` indicates an error such as identical tokens or an issue with sorting.
pub fn pair_for(e: Env, factory: Address, token_a: Address, token_b: Address) -> Result<Address, RaumFiRouterError> {
    let (token_0, token_1) = sort_tokens(token_a, token_b)?;
    let salt = pair_salt(&e, token_0, token_1);
    let deployer_with_address = e.deployer().with_address(factory.clone(), salt);
    let deterministic_address = deployer_with_address.deployed_address();
    Ok(deterministic_address)
}