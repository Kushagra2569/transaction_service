use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub fullname: String,
    pub email: String,
    pub role: String,
    pub token: String,
    pub balance: f64,
}

pub struct UserRegister {
    pub id: String,
    pub email: String,
    pub fullname: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub fullname: String,
    pub password: String,
    pub email: String,
    pub balance: Option<f64>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub password: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct UserAuth {
    pub email: String,
}

#[derive(Deserialize)]
pub struct TransactionRequest {
    pub from_email: String,
    pub to_email: String,
    pub amount: f64,
}

#[derive(Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub from_email: String,
    pub to_email: String,
    pub amount: f64,
    pub trnx_time: DateTime<Utc>,
}
