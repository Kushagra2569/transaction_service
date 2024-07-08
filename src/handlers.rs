use crate::config::db::get_conn;
use crate::errors::Errors;
use crate::utils::user_structs::LoginRequest;
use crate::utils::{
    user_controller::{login_user, register_user},
    user_structs::RegisterRequest,
};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
pub async fn fallback_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route {}", uri),
    )
}
//TODO: implement a server error function

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

pub async fn login_register(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    // Implement your user registration logic here
    let pool = get_conn().await;
    match login_user(&pool, &payload.email, &payload.password).await {
        Ok(user) => {
            let user_json = serde_json::json!({
                "fullname": user.fullname,
                "email": user.email,
            });
            (StatusCode::CREATED, Json(user_json))
        }
        Err(Errors::WrongCredentials) => {
            let error_json = serde_json::json!({
                "error": "Wrong credentials",
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
