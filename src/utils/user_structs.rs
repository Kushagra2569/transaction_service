use chrono::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub fullname: String,
    pub email: String,
    pub role: String,
    pub token: String,
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
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub password: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct UserBalance {
    pub email: String,
    pub balance: f64,
}
