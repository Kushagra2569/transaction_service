use super::service::authorize_user;
use crate::config::db::get_conn;
use crate::errors::Errors;
use crate::utils::user_controller::get_user_balance;
use crate::utils::{
    user_controller::{login_user, register_user},
    user_structs::{LoginRequest, RegisterRequest, UserAuth},
};
use axum::{
    extract::{Json, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
pub async fn fallback_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route {}", uri),
    )
}

pub async fn authorization_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("Authorization Middleware");
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
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
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if let Some(user_email) = authorize_user(extracted_token).await {
        req.extensions_mut().insert(user_email);
        Ok(next.run(req).await)
    } else {
        println!("Unauthorized");
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
    match register_user(&pool, &payload.fullname, &payload.email, &payload.password).await {
        Ok(user) => {
            let user_json = serde_json::json!({
                "fullname": user.fullname,
                "email": user.email,
            });
            (StatusCode::CREATED, Json(user_json))
        }
        Err(Errors::DuplicateUserEmail) => {
            let error_json = serde_json::json!({
                "error": "Email is already taken",
            });
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
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
            });
            (StatusCode::CREATED, Json(user_json))
        }
        Err(Errors::DuplicateUserEmail) => {
            let error_json = serde_json::json!({
                "error": "Email is already taken",
            });
            (StatusCode::BAD_REQUEST, Json(error_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
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
            (StatusCode::OK, Json(balance_json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string(),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}
