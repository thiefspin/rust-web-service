use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
    pub reset_token: Option<String>,
    pub reset_token_expires: Option<DateTime<Utc>>,
}

impl AuthUser {
    pub fn new(email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            is_active: true,
            is_verified: false,
            created_at: now,
            updated_at: now,
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            verification_token: Some(Uuid::new_v4().to_string()),
            reset_token: None,
            reset_token_expires: None,
        }
    }

    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    pub fn can_login(&self) -> bool {
        self.is_active && !self.is_locked()
    }

    pub fn increment_failed_attempts(&mut self) {
        self.failed_login_attempts += 1;

        // Lock account after 5 failed attempts for 15 minutes
        if self.failed_login_attempts >= 5 {
            self.locked_until = Some(Utc::now() + chrono::Duration::minutes(15));
        }

        self.updated_at = Utc::now();
    }

    pub fn reset_failed_attempts(&mut self) {
        self.failed_login_attempts = 0;
        self.locked_until = None;
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn generate_reset_token(&mut self) {
        self.reset_token = Some(Uuid::new_v4().to_string());
        self.reset_token_expires = Some(Utc::now() + chrono::Duration::hours(1));
        self.updated_at = Utc::now();
    }

    pub fn clear_reset_token(&mut self) {
        self.reset_token = None;
        self.reset_token_expires = None;
        self.updated_at = Utc::now();
    }

    pub fn is_reset_token_valid(&self, token: &str) -> bool {
        if let (Some(stored_token), Some(expires)) = (&self.reset_token, &self.reset_token_expires)
        {
            stored_token == token && Utc::now() < *expires
        } else {
            false
        }
    }

    pub fn verify_email(&mut self) {
        self.is_verified = true;
        self.verification_token = None;
        self.updated_at = Utc::now();
    }

    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
        self.clear_reset_token();
    }
}

// Request models
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    #[validate(custom = "validate_password_complexity")]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmResetPasswordRequest {
    pub token: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    #[validate(custom = "validate_password_complexity")]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    #[validate(custom = "validate_password_complexity")]
    pub new_password: String,
}

// Response models
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<AuthUser> for UserInfo {
    fn from(user: AuthUser) -> Self {
        Self {
            id: user.id,
            email: user.email,
            is_verified: user.is_verified,
            created_at: user.created_at,
            last_login: user.last_login,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

impl MessageResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: usize, // expiration time
    pub iat: usize, // issued at
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, expires_in_seconds: i64) -> Self {
        let now = Utc::now();
        let exp = (now + chrono::Duration::seconds(expires_in_seconds)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        Self {
            sub: user_id.to_string(),
            email,
            exp,
            iat,
        }
    }
}

// Password validation regex
// use once_cell::sync::Lazy;
// static PASSWORD_REGEX: Lazy<Regex> = Lazy::new(|| {
//     Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]").unwrap()
// });

use validator::ValidationError;
fn validate_password_complexity(pw: &str) -> Result<(), ValidationError> {
    if pw.len() < 8 {
        return Err(ValidationError::new(
            "Password must be at least 8 characters",
        ));
    }
    if !pw.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::new(
            "Password must contain a lowercase letter",
        ));
    }
    if !pw.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new(
            "Password must contain an uppercase letter",
        ));
    }
    if !pw.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("Password must contain a digit"));
    }
    if !pw.chars().any(|c| "@$!%*?&".contains(c)) {
        return Err(ValidationError::new(
            "Password must contain a special character (@$!%*?&)",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_creation() {
        let email = "test@example.com".to_string();
        let password_hash = "hashed_password".to_string();
        let user = AuthUser::new(email.clone(), password_hash.clone());

        assert_eq!(user.email, email);
        assert_eq!(user.password_hash, password_hash);
        assert!(user.is_active);
        assert!(!user.is_verified);
        assert_eq!(user.failed_login_attempts, 0);
        assert!(user.verification_token.is_some());
    }

    #[test]
    fn test_user_locking() {
        let mut user = AuthUser::new("test@example.com".to_string(), "hash".to_string());

        assert!(!user.is_locked());
        assert!(user.can_login());

        // Increment failed attempts
        for _ in 0..5 {
            user.increment_failed_attempts();
        }

        assert!(user.is_locked());
        assert!(!user.can_login());
    }

    #[test]
    fn test_reset_token_generation() {
        let mut user = AuthUser::new("test@example.com".to_string(), "hash".to_string());

        user.generate_reset_token();
        assert!(user.reset_token.is_some());
        assert!(user.reset_token_expires.is_some());

        let token = user.reset_token.as_ref().unwrap().clone();
        assert!(user.is_reset_token_valid(&token));
        assert!(!user.is_reset_token_valid("invalid_token"));
    }

    #[test]
    fn test_email_verification() {
        let mut user = AuthUser::new("test@example.com".to_string(), "hash".to_string());

        assert!(!user.is_verified);
        assert!(user.verification_token.is_some());

        user.verify_email();
        assert!(user.is_verified);
        assert!(user.verification_token.is_none());
    }

    #[test]
    fn test_claims_creation() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let expires_in = 3600;

        let claims = Claims::new(user_id, email.clone(), expires_in);

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert!(claims.exp > claims.iat);
    }
}
