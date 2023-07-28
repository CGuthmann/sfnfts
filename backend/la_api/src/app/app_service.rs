//! The service for the app controller

use num_bigint::BigInt;

//TODO: unsure about error handling
//  - forward errors from deep backend until they reach the api?
//      or return a error response as soon as possible?
//  - define api errors?

pub async fn get_nullifier() -> Result<Vec<BigInt>, &'static str> {
    Err("Not implemented!")
}

pub async fn get_trust_list() -> Result<Vec<BigInt>, &'static str> {
    Err("Not implemented!")
}

pub async fn get_merkle_tree() -> Result<Vec<BigInt>, &'static str> {
    Err("Not implemented!")
}

pub async fn post_mint() -> Result<(), &'static str> {
    Err("Not implemented!")
}

pub async fn post_transfer() -> Result<(), &'static str> {
    Err("Not implemented!")
}

pub async fn post_burn() -> Result<(), &'static str> {
    Err("Not implemented!")
}
