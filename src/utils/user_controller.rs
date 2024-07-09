use crate::errors::Errors;
use crate::service::{decode_token, encode_token};
use chrono::prelude::*;

use super::user_structs::{Transaction, User, UserRegister};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn register_user(
    pool: &PgPool,
    fullname: &str,
    email: &str,
    password: &str,
    balance: &f64,
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
    //TODO: check user exists
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
            "INSERT INTO users (id, full_name, role,email,balance) VALUES ($1, $2, 'user', $3,$4)",
        )
        .bind(&userlogin.id)
        .bind(&userlogin.fullname)
        .bind(&userlogin.email)
        .bind(&balance)
        .execute(pool)
        .await;

        if query2.is_ok() {
            let user = User {
                id: userlogin.id.clone(),
                fullname: userlogin.fullname.clone(),
                email: userlogin.email.clone(),
                role: "user".to_string(),
                token: "Not Valid".to_string(),
                balance: *balance,
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
        let balance = row.get::<String, &str>("balance");
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
                        balance: balance.parse::<f64>().unwrap(),
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

pub async fn create_transaction(
    pool: &PgPool,
    from_email: &str,
    to_email: &str,
    amount: f64,
) -> Result<Transaction, Errors> {
    let query1 = sqlx::query("SELECT balance FROM users WHERE email = $1")
        .bind(&from_email)
        .fetch_one(pool)
        .await;
    if query1.is_ok() {
        let row = query1.unwrap();
        let from_balance = row.get::<f64, &str>("balance");
        if from_balance < amount {
            let err = Errors::InsufficientBalance;
            return Err(err);
        }
        let query2 = sqlx::query("SELECT balance FROM users WHERE email = $1")
            .bind(&to_email)
            .fetch_one(pool)
            .await;
        if query2.is_ok() {
            let query3 = sqlx::query(
                "UPDATE users
            SET balance = CASE
                WHEN email = $1 THEN balance - $3
                WHEN email = $2 THEN balance + $3
            END
            WHERE email IN ($1, $2);",
            )
            .bind(from_email)
            .bind(to_email)
            .bind(amount)
            .execute(pool)
            .await;
            if query3.is_ok() {
                let id = Uuid::new_v4().as_simple().to_string();
                let trnx_time = Utc::now();
                let query4 = sqlx::query(
                    "INSERT INTO transactions (from_email, to_email, amount,id,created_at) VALUES ($1, $2, $3, $4,$5)",
                )
                .bind(&from_email)
                .bind(&to_email)
                .bind(&amount)
                .bind(&id)
                .bind(&trnx_time)
                .execute(pool)
                .await;
                if query4.is_ok() {
                    println!("Transaction successful");
                    let transaction = Transaction {
                        id,
                        from_email: from_email.to_string(),
                        to_email: to_email.to_string(),
                        amount,
                        trnx_time,
                    };
                    return Ok(transaction);
                } else {
                    //reverse the balance for failed transaction
                    println!(" transaction failed{:?}", query4);
                    let query5 = sqlx::query(
                        "UPDATE users
            SET balance = CASE
                WHEN email = $1 THEN balance + $3
                WHEN email = $2 THEN balance - $3
            END
            WHERE email IN ($1, $2);",
                    )
                    .bind(from_email)
                    .bind(to_email)
                    .bind(amount)
                    .execute(pool)
                    .await;
                    if query5.is_ok() {
                        let err = Errors::TransactionError;
                        return Err(err);
                    } else {
                        println!(
                            "Fatal error unable to reverse the account balance:{:?}",
                            query5
                        );
                        let err = Errors::DatabaseError(query5.err().unwrap());
                        return Err(err);
                    }
                }
            } else {
                println!("Unable to update account balance{:?}", query3);
                let err = Errors::TransactionError;
                return Err(err);
            }
        } else {
            let err = Errors::UserDoesNotExist;
            return Err(err);
        }
    } else {
        let err = Errors::DatabaseError(query1.err().unwrap());
        return Err(err);
    }
}
