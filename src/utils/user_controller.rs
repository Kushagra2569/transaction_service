use crate::errors::Errors;
use crate::service::{decode_token, encode_token};
use chrono::prelude::*;

use super::user_structs::{User, UserRegister};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn register_user(
    pool: &PgPool,
    fullname: &str,
    email: &str,
    password: &str,
) -> Result<User, Errors> {
    let password_hash = hash(password, DEFAULT_COST).unwrap();
    let userlogin = &UserRegister {
        id: Uuid::new_v4().as_simple().to_string(),
        fullname: fullname.to_string(),
        email: email.to_string(),
        password_hash,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let query1 = sqlx::query(
        "INSERT INTO userlogin (full_name, password, email, id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&userlogin.fullname)
    .bind(&userlogin.password_hash)
    .bind(&userlogin.email)
    .bind(&userlogin.id)
    .bind(&userlogin.created_at)
    .bind(&userlogin.updated_at)
    .execute(pool)
    .await;

    if query1.is_ok() {
        let query2 = sqlx::query(
            "INSERT INTO users (id, full_name, role,email) VALUES ($1, $2, 'user', $3)",
        )
        .bind(&userlogin.id)
        .bind(&userlogin.fullname)
        .bind(&userlogin.email)
        .execute(pool)
        .await;

        if query2.is_ok() {
            let user = User {
                id: userlogin.id.clone(),
                fullname: userlogin.fullname.clone(),
                email: userlogin.email.clone(),
                role: "user".to_string(),
                token: "Not Valid".to_string(),
            };
            return Ok(user);
        } else {
            let _ = sqlx::query("DELETE FROM userlogin WHERE id = $1")
                .bind(&userlogin.id)
                .execute(pool)
                .await;

            println!("{:?}", query2);

            let err = Errors::DatabaseError(query2.unwrap_err());
            return Err(err);
        }
    } else {
        println!("{:?}", query1);
        let err = Errors::DatabaseError(query1.unwrap_err());
        return Err(err);
    }
}
pub async fn login_user(pool: &PgPool, email: &str, password: &str) -> Result<User, Errors> {
    let query1 = sqlx::query("SELECT * FROM userlogin WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await;

    if query1.is_ok() {
        let row = query1.unwrap();
        let pass = row.get::<String, &str>("password");
        let userid = row.get::<String, &str>("id");
        let email = row.get::<String, &str>("email");
        let fullname = row.get::<String, &str>("full_name");
        println!("id: {}", userid);
        let dehashed_pass = verify(password, &pass);
        match dehashed_pass {
            Ok(value) => {
                if value {
                    println!("Password verified: {}", value);
                    let token = encode_token(email.clone());
                    let tokenstr;
                    if token.is_err() {
                        let err = Errors::InternalServerError;
                        return Err(err);
                    } else {
                        tokenstr = token.unwrap();
                    }

                    let query2 = sqlx::query("INSERT INTO authorise (id, email, token) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET token = EXCLUDED.token")
                        .bind(&userid)
                        .bind(&email)
                        .bind(&tokenstr)
                        .execute(pool)
                        .await;
                    if query2.is_err() {
                        println!("{:?}", query2);
                        let err = Errors::InternalServerError;
                        return Err(err);
                    }
                    let user = User {
                        id: userid.to_string(),
                        fullname: fullname.to_string(),
                        email: email.to_string(),
                        role: "user".to_string(),
                        token: tokenstr,
                        //TODO: get role from db
                    };
                    println!("User: {} logged in at {}", user.email, Utc::now());
                    return Ok(user);
                } else {
                    let err = Errors::WrongCredentials;
                    return Err(err);
                }
            }
            Err(err) => {
                println!("Password verification error: {}", err);
                let err = Errors::BcryptError(err);
                return Err(err);
            }
        }
    } else {
        let err = Errors::WrongCredentials;
        return Err(err);
    }
}

pub async fn get_user_balance(pool: &PgPool, email: &str) -> Result<f64, Errors> {
    let query = sqlx::query("SELECT balance FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await;

    if query.is_ok() {
        let row = query.unwrap();
        let balance = row.get::<f64, &str>("balance");
        return Ok(balance);
    } else {
        let err = Errors::DatabaseError(query.err().unwrap());
        return Err(err);
    }
}
