use crate::{config::db::get_conn, errors::Errors};
use chrono::{prelude::*, Duration};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
}

pub fn encode_token(payload: String) -> Result<String, Errors> {
    dotenv().ok();
    let secret = env::var("JWT_KEY").expect("SECRET must be set");
    let time_now = Utc::now();
    let expire = Duration::hours(24);
    let exp: usize = (time_now + expire).timestamp() as usize;
    let iat: usize = time_now.timestamp() as usize;
    let user_claim = Claims {
        exp,
        iat,
        email: payload,
    };
    encode(
        &Header::default(),
        &user_claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| Errors::InternalServerError)
}

pub fn decode_token(token: &str) -> Result<TokenData<Claims>, Errors> {
    dotenv().ok();
    let secret = env::var("JWT_KEY").expect("SECRET must be set");
    let result: Result<TokenData<Claims>, Errors> = decode(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| Errors::InternalServerError);
    result
}

pub async fn authorize_user(token: &str) -> Option<String> {
    match decode_token(token) {
        Ok(token_data) => {
            let pool = get_conn().await;
            let query = sqlx::query("SELECT * FROM authorise WHERE token = $1")
                .bind(&token)
                .fetch_optional(pool)
                .await;
            if query.is_ok() {
                let email = token_data.claims.email.clone();
                Some(email)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
