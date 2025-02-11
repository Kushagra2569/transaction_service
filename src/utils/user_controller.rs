use crate::errors::Errors;
use crate::service::encode_token;
use chrono::prelude::*;

use super::user_structs::{Transaction, User, UserRegister};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::{PgPool, Row};
use tracing::{error, info, warn};
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
    let query = sqlx::query("SELECT * FROM userlogin WHERE email = $1")
        .bind(&userlogin.email)
        .fetch_optional(pool)
        .await;
    if query.is_ok() {
        let row = query.unwrap();
        if row.is_some() {
            warn!(
                "Registering User with email {} already exists",
                userlogin.email
            );
            let err = Errors::DuplicateUserEmail;
            return Err(err);
        }
    }
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

            error!("Unable to insert into users table{:?}", query2);

            let err = Errors::DatabaseError(query2.unwrap_err());
            return Err(err);
        }
    } else {
        error!("Unable to insert into userlogin table{:?}", query1);
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
        let dehashed_pass = verify(password, &pass);
        match dehashed_pass {
            Ok(value) => {
                if value {
                    let token = encode_token(email.clone());
                    let tokenstr;
                    if token.is_err() {
                        error!("Unable to generate token{:?}", token);
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
                        error!("Unable to insert into authorise table{:?}", query2);
                        let err = Errors::InternalServerError;
                        return Err(err);
                    }
                    let query3 = sqlx::query("SELECT balance FROM users WHERE id = $1")
                        .bind(&userid)
                        .fetch_one(pool)
                        .await;
                    if query3.is_err() {
                        error!("Unable to get balance for user{:?}", email);
                        let err = Errors::DatabaseError(query3.err().unwrap());
                        return Err(err);
                    }
                    let row3 = query3.unwrap();
                    let balance = row3.get::<f64, &str>("balance");
                    let user = User {
                        id: userid.to_string(),
                        fullname: fullname.to_string(),
                        email: email.to_string(),
                        role: "user".to_string(),
                        token: tokenstr,
                        balance,
                        //TODO: get role from db
                    };
                    info!("User: {} logged in at {}", user.email, Utc::now());
                    return Ok(user);
                } else {
                    let err = Errors::WrongCredentials;
                    return Err(err);
                }
            }
            Err(err) => {
                error!("Password verification error: {}", err);
                let err = Errors::BcryptError(err);
                return Err(err);
            }
        }
    } else {
        warn!("User with email {} does not exist", email);
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
        error!("Unable to get balance");
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
            warn!("user {} has insufficient balance", from_email);
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
                    error!(" transaction failed{:?}", query4);
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
                        error!(
                            "FATAL: error unable to reverse the account balance:{:?}",
                            query5
                        );
                        let err = Errors::DatabaseError(query5.err().unwrap());
                        return Err(err);
                    }
                }
            } else {
                error!("Unable to update account balance{:?}", query3);
                let err = Errors::TransactionError;
                return Err(err);
            }
        } else {
            error!("User with email {} does not exist", to_email);
            let err = Errors::UserDoesNotExist;
            return Err(err);
        }
    } else {
        error!("User with email {} does not exist", to_email);
        let err = Errors::UserDoesNotExist;
        return Err(err);
    }
}

pub async fn list_transactions(pool: &PgPool, email: &str) -> Result<Vec<Transaction>, Errors> {
    let query = sqlx::query("SELECT * FROM transactions WHERE from_email = $1 OR to_email = $1")
        .bind(email)
        .fetch_all(pool)
        .await;
    if query.is_ok() {
        let mut transactions = Vec::new();
        for row in query.unwrap() {
            let id = row.get::<String, &str>("id");
            let from_email = row.get::<String, &str>("from_email");
            let to_email = row.get::<String, &str>("to_email");
            let amount = row.get::<f64, &str>("amount");
            let trnx_time = row.get::<DateTime<Utc>, &str>("created_at");
            let transaction = Transaction {
                id,
                from_email,
                to_email,
                amount,
                trnx_time,
            };
            transactions.push(transaction);
        }
        return Ok(transactions);
    } else {
        error!("Unable to get transactions");
        let err = Errors::DatabaseError(query.err().unwrap());
        return Err(err);
    }
}

pub async fn update_user(
    pool: &PgPool,
    user_email: &str,
    old_name: &str,
    new_name: &str,
) -> Result<(), Errors> {
    let query1 = sqlx::query("SELECT * FROM users WHERE email = $1")
        .bind(user_email)
        .fetch_one(pool)
        .await;
    if query1.is_ok() {
        let row = query1.unwrap();
        let fullname = row.get::<String, &str>("full_name");
        if fullname != old_name {
            let err = Errors::WrongCredentials;
            return Err(err);
        }
        let query2 = sqlx::query("UPDATE users SET full_name = $1 WHERE email = $2")
            .bind(new_name)
            .bind(user_email)
            .execute(pool)
            .await;
        if query2.is_ok() {
            let query3 = sqlx::query("UPDATE userlogin SET full_name = $1 WHERE email = $2")
                .bind(new_name)
                .bind(user_email)
                .execute(pool)
                .await;
            if query3.is_ok() {
                return Ok(());
            } else {
                let query4 = sqlx::query("UPDATE users SET full_name = $1 WHERE email = $2")
                    .bind(old_name)
                    .bind(user_email)
                    .execute(pool)
                    .await;
                if query4.is_err() {
                    error!("FATAL:Unable to revert users table{:?}", query4);
                }
                let err = Errors::DatabaseError(query3.err().unwrap());
                return Err(err);
            }
        } else {
            error!("Unable to update users table{:?}", query2);
            let err = Errors::DatabaseError(query2.err().unwrap());
            return Err(err);
        }
    } else {
        error!(" Unable to find user {} ", user_email);
        let err = Errors::DatabaseError(query1.err().unwrap());
        return Err(err);
    }
}
