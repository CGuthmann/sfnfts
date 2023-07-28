//! The main controller for "/" routes.

use num_bigint::BigInt;
use rocket::{get, post, head};
use rocket::http::Status;
use rocket::serde::json::Json;

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
            Err((Status::NotFound, "There is no nullifier entry in the database."))
        } else {
            Ok(Json(val))
        }
    } else {
        Err((Status::InternalServerError, "Server couldn't query nullifier list."))
    }
}

#[get("/trust_list")]
pub async fn get_trust_list() -> Result<Json<Vec<BigInt>>, (Status, &'static str)> {
    if let Ok(val) = app_service::get_trust_list().await {
        if val.is_empty() {
            Err((Status::NotFound, "There is no trust list entry in the database."))
        } else {
            Ok(Json(val))
        }
    } else {
        Err((Status::InternalServerError, "Server couldn't query trust list"))
    }
}

#[get("/merkle_tree")]
pub async fn get_merkle_tree() -> Result<Json<Vec<BigInt>>, (Status, &'static str)> {
    if let Ok(val) = app_service::get_merkle_tree().await {
        if val.is_empty() {
            Err((Status::NotFound, "There is no merkle tree entry in the database."))
        } else {
            Ok(Json(val))
        }
    } else {
        Err((Status::InternalServerError, "Server couldn't query merkle tree."))
    }
}

//TODO: Input (DTO)
#[post("/mint")]
pub async fn post_mint() -> Result<Status, Status> {
    if (app_service::post_mint().await).is_ok() {
        Ok(Status::Ok)
    } else {
        Err(Status::InternalServerError)
    }
}

//TODO: Input (DTO)
#[post("/transfer")]
pub async fn post_transfer() -> Result<Status, Status> {
    if (app_service::post_transfer().await).is_ok() {
        Ok(Status::Ok)
    } else {
        Err(Status::InternalServerError)
    }
}

//TODO: Input (DTO)
#[post("/burn")]
pub async fn post_burn() -> Result<Status, Status> {
    if (app_service::post_burn().await).is_ok() {
        Ok(Status::Ok)
    } else {
        Err(Status::InternalServerError)
    }
}
