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
    #[error("internal server error")]
    InternalServerError,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("User Does not exist")]
    UserDoesNotExist,
    #[error("Unable to create transaction")]
    TransactionError,
}
