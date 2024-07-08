use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Errors {
    #[error(transparent)]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("wrong credentials")]
    WrongCredentials,
    #[error("email is already taken")]
    DuplicateUserEmail,
}
