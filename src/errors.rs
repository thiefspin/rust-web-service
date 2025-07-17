use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Authentication failed")]
    Unauthorized,

    #[error("Access denied")]
    Forbidden,

    #[error("Resource not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Database error")]
    DatabaseError,

    #[error("Internal server error")]
    InternalError,

    #[error("Password hashing error")]
    PasswordHashError,

    #[error("JWT error: {0}")]
    JwtError(String),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: self.to_string(),
            }),
            ServiceError::Forbidden => HttpResponse::Forbidden().json(ErrorResponse {
                error: "forbidden".to_string(),
                message: self.to_string(),
            }),
            ServiceError::NotFound => HttpResponse::NotFound().json(ErrorResponse {
                error: "not_found".to_string(),
                message: self.to_string(),
            }),
            ServiceError::BadRequest(_) => HttpResponse::BadRequest().json(ErrorResponse {
                error: "bad_request".to_string(),
                message: self.to_string(),
            }),
            ServiceError::ValidationError(_) => HttpResponse::BadRequest().json(ErrorResponse {
                error: "validation_error".to_string(),
                message: self.to_string(),
            }),
            ServiceError::UserAlreadyExists => HttpResponse::Conflict().json(ErrorResponse {
                error: "user_already_exists".to_string(),
                message: self.to_string(),
            }),
            ServiceError::InvalidCredentials => HttpResponse::Unauthorized().json(ErrorResponse {
                error: "invalid_credentials".to_string(),
                message: self.to_string(),
            }),
            ServiceError::TokenExpired => HttpResponse::Unauthorized().json(ErrorResponse {
                error: "token_expired".to_string(),
                message: self.to_string(),
            }),
            ServiceError::InvalidToken => HttpResponse::Unauthorized().json(ErrorResponse {
                error: "invalid_token".to_string(),
                message: self.to_string(),
            }),
            ServiceError::DatabaseError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "database_error".to_string(),
                    message: "A database error occurred".to_string(),
                })
            }
            ServiceError::InternalError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "An internal server error occurred".to_string(),
                })
            }
            ServiceError::PasswordHashError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "password_hash_error".to_string(),
                    message: "Password processing failed".to_string(),
                })
            }
            ServiceError::JwtError(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: "jwt_error".to_string(),
                message: "Token processing failed".to_string(),
            }),
        }
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// Convert from various error types to ServiceError
impl From<sqlx::Error> for ServiceError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => ServiceError::NotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => ServiceError::UserAlreadyExists, // PostgreSQL unique violation
                        _ => ServiceError::DatabaseError,
                    }
                } else {
                    ServiceError::DatabaseError
                }
            }
            _ => ServiceError::DatabaseError,
        }
    }
}

impl From<bcrypt::BcryptError> for ServiceError {
    fn from(_: bcrypt::BcryptError) -> Self {
        ServiceError::PasswordHashError
    }
}

impl From<jsonwebtoken::errors::Error> for ServiceError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ServiceError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => ServiceError::InvalidToken,
            _ => ServiceError::JwtError(error.to_string()),
        }
    }
}

impl From<validator::ValidationErrors> for ServiceError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let error_messages: Vec<String> = errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let field_errors: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .unwrap_or(&std::borrow::Cow::Borrowed("Invalid value"))
                            .to_string()
                    })
                    .collect();
                format!("{}: {}", field, field_errors.join(", "))
            })
            .collect();

        ServiceError::ValidationError(error_messages.join("; "))
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
