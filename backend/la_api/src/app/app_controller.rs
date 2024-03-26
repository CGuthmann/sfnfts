//! The main controller for "/" routes.
use std::hash::Hash;

use crypto::merkle_tree::HashTree;
use crypto::poseidon::PoseidonHasher;
use num_bigint::BigInt;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, head, post};
use serde::Deserialize;

use super::app_service;

//TODO: unsure about return types.
//  - use results or just return status some times?
//  - define catchers and response builders?
//  - define type for "Error-Status with Reason" (Status, &str)?

#[get("/")]
pub fn get_root() -> Result<(), Status> {
    Err(Status::NotImplemented)
}

#[head("/")]
pub fn head_root() -> Result<(), Status> {
    Err(Status::NotImplemented)
}

//TODO: Json<_> might not implement response generation
#[get("/nullifier")]
pub async fn get_nullifier() -> Result<Json<Vec<BigInt>>, (Status, &'static str)> {
    if let Ok(val) = app_service::get_nullifier().await {
        if val.is_empty() {
            Err((
                Status::NotFound,
                "There is no nullifier entry in the database.",
            ))
        } else {
            Ok(Json(val))
        }
    } else {
        Err((
            Status::InternalServerError,
            "Server couldn't query nullifier list.",
        ))
    }
}

#[get("/trust_list")]
pub async fn get_trust_list() -> Result<Json<HashTree<PoseidonHasher,2>>, (Status, &'static str)> {
    if let Ok(val) = app_service::get_trust_list().await {
        if val.leaves.is_empty() {
            Err((
                Status::NotFound,
                "There is no trust list entry in the database.",
            ))
        } else {
            Ok(Json(val))
        }
    } else {
        Err((
            Status::InternalServerError,
            "Server couldn't query trust list",
        ))
    }
}

#[get("/merkle_tree")]
pub async fn get_merkle_tree() -> Result<Json<Vec<HashTree<PoseidonHasher,2>>>, (Status, &'static str)> {
    if let Ok(val) = app_service::get_merkle_tree().await {
        if val.is_empty() {
            Err((
                Status::NotFound,
                "There is no merkle tree entry in the database.",
            ))
        } else {
            Ok(Json(val))
        }
    } else {
        Err((
            Status::InternalServerError,
            "Server couldn't query merkle tree.",
        ))
    }
}

#[post("/mint", format = "json", data = "<proof>")]
pub async fn post_mint(
    proof: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>,
) -> Result<Json<(usize,usize)>, Status> {
    let index = app_service::post_mint(proof).await;
    match index {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}


#[post("/transfer", format = "json", data = "<proof>")]
pub async fn post_transfer(
    proof: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>,
) -> Result<Json<((usize,usize),(usize,usize))>, Status> {
    let index = app_service::post_transfer(proof).await;
    match index {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}


#[post("/burn", format = "json", data = "<proof>")]
pub async fn post_burn(
    proof: Json<ark_circom_service::ArkCircomFullProof<ark_circom_service::Bn254>>,
) -> Result<Json<(usize,usize)>, Status> {
    let status= app_service::post_burn(proof).await;
    dbg!(&status);
    match status {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
