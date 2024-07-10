use super::service::authorize_user;
use crate::config::db::get_conn;
use crate::errors::Errors;
use crate::utils::{
    user_controller::{
        create_transaction, get_user_balance, list_transactions, login_user, register_user,
        update_user,
    },
    user_structs::{LoginRequest, ModifyUser, RegisterRequest, TransactionRequest, UserAuth},
};
use axum::Extension;
use axum::{
    extract::{Json, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{error, info, instrument, warn};

/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
#[instrument]
pub async fn fallback_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        warn!("No route {}", uri),
        format!("No route {}", uri),
    )
}

pub async fn authorization_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        warn!("No Authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    };
    let mut header = auth_header.split_whitespace();
    let (_bearer, token) = (header.next(), header.next());
    let extracted_token;
    match token {
        Some(newtoken) => {
            extracted_token = newtoken;
        }
        None => {
            warn!("Invalid token");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if let Some(user_email) = authorize_user(extracted_token).await {
        req.extensions_mut().insert(user_email);
        Ok(next.run(req).await)
    } else {
        warn!("Unauthorized");
        Err(StatusCode::UNAUTHORIZED)
    }
}
pub async fn authorise_check(req: Request) -> impl IntoResponse {
    let user_email = req.extensions().get::<String>().unwrap();
    let user_json = serde_json::json!({
        "email": user_email,
    });
    (StatusCode::OK, Json(user_json))
}

pub async fn register_handler(Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    // Implement your user registration logic here
    let pool = get_conn().await;
    let mut initial_balance = 0.0;
    if let Some(balance) = payload.balance {
        if balance < 0.0 {
            let error_json = serde_json::json!({
                "error": "Balance cannot be negative",
            });
            info!(
                "user: {} tried to register with amount {} with negative balance",
                payload.email, balance
            );
            return (StatusCode::BAD_REQUEST, Json(error_json));
        }
        initial_balance = balance;
    }
    match register_user(
        &pool,
        &payload.fullname,
        &payload.email,
        &payload.password,
        &initial_balance,
    )
    .await
    {
        Ok(user) => {
            let user_json = serde_json::json!({
                "fullname": user.fullname,
                "email": user.email,
            });
            info!("user: {} registered successfully", user.email);
            (StatusCode::CREATED, Json(user_json))
        }
        Err(Errors::DuplicateUserEmail) => {
            let error_json = serde_json::json!({
                "error": "Email is already taken",
            });
            warn!(
                "user: {} attempted to register with an existing email",
                payload.email
            );
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            error!("error occurred while registering user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}

pub async fn login_handler(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    // Implement your user registration logic here
    let pool = get_conn().await;
    match login_user(&pool, &payload.email, &payload.password).await {
        Ok(user) => {
            // let verified_token = decode_token(token.unwrap().as_str());
            let user_json = serde_json::json!({
                "fullname": user.fullname,
                "email": user.email,
                "token" : user.token,
                "balance": user.balance,
            });
            info!("user: {} logged in successfully", user.email);
            (StatusCode::CREATED, Json(user_json))
        }
        Err(Errors::WrongCredentials) => {
            let error_json = serde_json::json!({
                "error": "Wrong credentials",
            });
            info!(
                "user: {} attempted to login with wrong credentials",
                payload.email
            );
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            error!("error occurred while logging in user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}

pub async fn user_balance_handler(Json(payload): Json<UserAuth>) -> impl IntoResponse {
    let pool = get_conn().await;
    let user_email = payload.email.clone();

    match get_user_balance(pool, user_email.as_str()).await {
        Ok(balance) => {
            let balance_json = serde_json::json!({
                "balance": balance,
                "email": user_email.as_str(),
            });
            info!("user: {} checked balance successfully", user_email);
            (StatusCode::OK, Json(balance_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            error!("error occurred while checking balance: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}

pub async fn create_transaction_handler(
    Extension(user_email): Extension<String>,
    Json(payload): Json<TransactionRequest>,
) -> impl IntoResponse {
    let pool = get_conn().await;
    let from_email = payload.from_email.clone();
    let to_email = payload.to_email.clone();
    let amount = payload.amount;

    if amount < 0.0 {
        let error_json = serde_json::json!({
            "error": "Amount cannot be negative",
        });
        warn!(
            "user: {} initiated transaction amount {} with negative amount",
            from_email, amount
        );
        return (StatusCode::BAD_REQUEST, Json(error_json));
    }

    if from_email == to_email {
        let error_json = serde_json::json!({
            "error": "Cannot transfer to self",
        });
        warn!("user: {} initiated transaction to self", from_email);
        return (StatusCode::BAD_REQUEST, Json(error_json));
    }

    if user_email != from_email {
        let error_json = serde_json::json!({
            "error": "Unauthorized",
        });
        warn!(
            "user: {} attempted to initiate transaction for another user: {}",
            user_email, from_email
        );
        return (StatusCode::UNAUTHORIZED, Json(error_json));
    }

    match create_transaction(pool, from_email.as_str(), to_email.as_str(), amount).await {
        Ok(transaction) => {
            let transaction_json = serde_json::json!({
                "from_email": transaction.from_email,
                "to_email": transaction.to_email,
                "amount": transaction.amount,
            });
            info!(
                "user: {} initiated transaction to user: {} with amount {}",
                from_email, to_email, amount
            );
            (StatusCode::CREATED, Json(transaction_json))
        }
        Err(Errors::InsufficientBalance) => {
            let error_json = serde_json::json!({
                "error": "Insufficient balance",
            });
            warn!(
                "user: {} attempted to initiate transaction with insufficient balance",
                from_email
            );
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(Errors::UserDoesNotExist) => {
            let error_json = serde_json::json!({
                "error": "User does not exist",
            });
            warn!(
                "user: {} attempted to initiate transaction to non-existent user: {}",
                from_email, to_email
            );
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(Errors::TransactionError) => {
            let error_json = serde_json::json!({
                "error": "Transaction error",
            });
            error!("error occurred while creating transaction");
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            error!("error occurred while creating transaction: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}

pub async fn list_transaction_handler(
    Extension(user_email): Extension<String>,
) -> impl IntoResponse {
    let pool = get_conn().await;
    match list_transactions(pool, user_email.as_str()).await {
        Ok(transactions) => {
            let transactions_json = serde_json::json!({
                "transactions": transactions,
            });
            info!("user: {} listed transactions successfully", user_email);
            (StatusCode::OK, Json(transactions_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            error!("error occurred while listing transactions: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}

pub async fn modify_user_handler(
    Extension(user_email): Extension<String>,
    Json(payload): Json<ModifyUser>,
) -> impl IntoResponse {
    let pool = get_conn().await;
    let old_name = payload.old_name.clone();
    let new_name = payload.new_name.clone();

    match update_user(
        pool,
        user_email.as_str(),
        old_name.as_str(),
        new_name.as_str(),
    )
    .await
    {
        Ok(()) => {
            let user_json = serde_json::json!({
                "fullname": new_name,
                "email": user_email,
            });
            info!(
                "user: {} updated fullname from {} to {}",
                user_email, old_name, new_name
            );
            (StatusCode::CREATED, Json(user_json))
        }
        Err(Errors::WrongCredentials) => {
            let error_json = serde_json::json!({
                "error": "Old fullname does not match user's fullname",
            });
            warn!(
                "user: {} attempted to update fullname with wrong old name: {}",
                user_email, old_name
            );
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            error!("error occurred while updating user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}
