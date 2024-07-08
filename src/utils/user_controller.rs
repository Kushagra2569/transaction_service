use crate::errors::Errors;
use chrono::prelude::*;

use super::user_structs::{User, UserLogin, UserRegister};
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
        let query2 = sqlx::query("INSERT INTO users (id, full_name, role) VALUES ($1, $2, 'user')")
            .bind(&userlogin.id)
            .bind(&userlogin.fullname)
            .execute(pool)
            .await;

        if query2.is_ok() {
            let user = User {
                id: userlogin.id.clone(),
                fullname: userlogin.fullname.clone(),
                email: userlogin.email.clone(),
                role: "user".to_string(),
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
        println!("id: {}", userid);
        let dehashed_pass = verify(password, &pass);
        match dehashed_pass {
            Ok(value) => {
                if value {
                    println!("Password verified: {}", value);
                    let query2 = sqlx::query("SELECT * FROM users WHERE id = ($1)")
                        .bind(&userid)
                        .fetch_one(pool)
                        .await;
                    if query2.is_err() {
                        let err = Errors::WrongCredentials;
                        return Err(err);
                    }
                    let row2 = query2.unwrap();
                    let user = User {
                        id: row2.get::<String, &str>("id"),
                        fullname: row2.get::<String, &str>("full_name"),
                        email: email.to_string(),
                        role: "user".to_string(),
                        //TODO: get role from db
                    };
                    println!("User: {} logged it at {}", user.email, Utc::now());
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
