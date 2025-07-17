use bcrypt::{hash, verify};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::{
    config::CONFIG,
    errors::{ServiceError, ServiceResult},
    middleware::auth::generate_jwt_token,
    models::auth_user::{
        AuthResponse, AuthUser, ChangePasswordRequest, ConfirmResetPasswordRequest, LoginRequest,
        MessageResponse, RegisterRequest, ResetPasswordRequest, UserInfo, VerifyEmailRequest,
    },
};

#[derive(Clone)]
pub struct AuthService {
    db_pool: Pool<Postgres>,
}

impl AuthService {
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        Self { db_pool }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> ServiceResult<MessageResponse> {
        // Check if user already exists
        if self.get_user_by_email(&request.email).await.is_ok() {
            return Err(ServiceError::UserAlreadyExists);
        }

        // Hash the password
        let password_hash = hash(&request.password, CONFIG.bcrypt_cost)?;

        // Create new user
        let user = AuthUser::new(request.email, password_hash);

        // Insert user into database
        sqlx::query(
            r#"
            INSERT INTO auth_users (
                id, email, password_hash, is_active, is_verified,
                created_at, updated_at, failed_login_attempts,
                verification_token
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.is_active)
        .bind(user.is_verified)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.failed_login_attempts)
        .bind(&user.verification_token)
        .execute(&self.db_pool)
        .await?;

        Ok(MessageResponse::new(
            "User registered successfully. Please check your email for verification.",
        ))
    }

    /// Login a user
    pub async fn login(&self, request: LoginRequest) -> ServiceResult<AuthResponse> {
        let mut user = self.get_user_by_email(&request.email).await?;

        // Check if user can login (not locked, active)
        if !user.can_login() {
            return Err(ServiceError::Unauthorized);
        }

        // Verify password
        if !verify(&request.password, &user.password_hash)? {
            // Increment failed attempts
            user.increment_failed_attempts();
            self.update_user_login_attempts(&user).await?;
            return Err(ServiceError::InvalidCredentials);
        }

        // Reset failed attempts and update last login
        user.reset_failed_attempts();
        self.update_user_successful_login(&user).await?;

        // Generate JWT token
        let access_token = generate_jwt_token(user.id, user.email.clone())?;

        Ok(AuthResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: CONFIG.jwt_expiration,
            user: UserInfo::from(user),
        })
    }

    /// Verify email address
    pub async fn verify_email(
        &self,
        request: VerifyEmailRequest,
    ) -> ServiceResult<MessageResponse> {
        let mut user = self.get_user_by_verification_token(&request.token).await?;

        if user.verification_token.as_ref() != Some(&request.token) {
            return Err(ServiceError::InvalidToken);
        }

        user.verify_email();
        self.update_user_verification(&user).await?;

        Ok(MessageResponse::new("Email verified successfully."))
    }

    /// Request password reset
    pub async fn request_password_reset(
        &self,
        request: ResetPasswordRequest,
    ) -> ServiceResult<MessageResponse> {
        let mut user = match self.get_user_by_email(&request.email).await {
            Ok(user) => user,
            Err(ServiceError::NotFound) => {
                // Don't reveal if email exists or not
                return Ok(MessageResponse::new(
                    "If the email exists, a password reset link has been sent.",
                ));
            }
            Err(e) => return Err(e),
        };

        user.generate_reset_token();
        self.update_user_reset_token(&user).await?;

        // In a real application, you would send an email here
        // For now, we'll just return the token (don't do this in production!)
        log::info!(
            "Password reset token for {}: {}",
            user.email,
            user.reset_token.as_ref().unwrap()
        );

        Ok(MessageResponse::new(
            "If the email exists, a password reset link has been sent.",
        ))
    }

    /// Confirm password reset
    pub async fn confirm_password_reset(
        &self,
        request: ConfirmResetPasswordRequest,
    ) -> ServiceResult<MessageResponse> {
        let mut user = self.get_user_by_reset_token(&request.token).await?;

        if !user.is_reset_token_valid(&request.token) {
            return Err(ServiceError::InvalidToken);
        }

        // Hash new password
        let new_password_hash = hash(&request.new_password, CONFIG.bcrypt_cost)?;
        user.update_password(new_password_hash);

        self.update_user_password(&user).await?;

        Ok(MessageResponse::new("Password reset successfully."))
    }

    /// Change password (for authenticated users)
    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> ServiceResult<MessageResponse> {
        let mut user = self.get_user_by_id(user_id).await?;

        // Verify current password
        if !verify(&request.current_password, &user.password_hash)? {
            return Err(ServiceError::InvalidCredentials);
        }

        // Hash new password
        let new_password_hash = hash(&request.new_password, CONFIG.bcrypt_cost)?;
        user.update_password(new_password_hash);

        self.update_user_password(&user).await?;

        Ok(MessageResponse::new("Password changed successfully."))
    }

    /// Get user info (for authenticated users)
    pub async fn get_user_info(&self, user_id: Uuid) -> ServiceResult<UserInfo> {
        let user = self.get_user_by_id(user_id).await?;
        Ok(UserInfo::from(user))
    }

    /// Refresh JWT token
    pub async fn refresh_token(&self, user_id: Uuid) -> ServiceResult<AuthResponse> {
        let user = self.get_user_by_id(user_id).await?;

        if !user.can_login() {
            return Err(ServiceError::Unauthorized);
        }

        let access_token = generate_jwt_token(user.id, user.email.clone())?;

        Ok(AuthResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: CONFIG.jwt_expiration,
            user: UserInfo::from(user),
        })
    }

    // Private helper methods
    async fn get_user_by_email(&self, email: &str) -> ServiceResult<AuthUser> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, is_active, is_verified,
                   created_at, updated_at, last_login, failed_login_attempts,
                   locked_until, verification_token, reset_token, reset_token_expires
            FROM auth_users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_one(&self.db_pool)
        .await?;

        let user = AuthUser {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            verification_token: row.get("verification_token"),
            reset_token: row.get("reset_token"),
            reset_token_expires: row.get("reset_token_expires"),
        };

        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> ServiceResult<AuthUser> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, is_active, is_verified,
                   created_at, updated_at, last_login, failed_login_attempts,
                   locked_until, verification_token, reset_token, reset_token_expires
            FROM auth_users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await?;

        let user = AuthUser {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            verification_token: row.get("verification_token"),
            reset_token: row.get("reset_token"),
            reset_token_expires: row.get("reset_token_expires"),
        };

        Ok(user)
    }

    async fn get_user_by_verification_token(&self, token: &str) -> ServiceResult<AuthUser> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, is_active, is_verified,
                   created_at, updated_at, last_login, failed_login_attempts,
                   locked_until, verification_token, reset_token, reset_token_expires
            FROM auth_users
            WHERE verification_token = $1
            "#,
        )
        .bind(token)
        .fetch_one(&self.db_pool)
        .await?;

        let user = AuthUser {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            verification_token: row.get("verification_token"),
            reset_token: row.get("reset_token"),
            reset_token_expires: row.get("reset_token_expires"),
        };

        Ok(user)
    }

    async fn get_user_by_reset_token(&self, token: &str) -> ServiceResult<AuthUser> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, is_active, is_verified,
                   created_at, updated_at, last_login, failed_login_attempts,
                   locked_until, verification_token, reset_token, reset_token_expires
            FROM auth_users
            WHERE reset_token = $1
            "#,
        )
        .bind(token)
        .fetch_one(&self.db_pool)
        .await?;

        let user = AuthUser {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            verification_token: row.get("verification_token"),
            reset_token: row.get("reset_token"),
            reset_token_expires: row.get("reset_token_expires"),
        };

        Ok(user)
    }

    async fn update_user_login_attempts(&self, user: &AuthUser) -> ServiceResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_users
            SET failed_login_attempts = $1, locked_until = $2, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(user.failed_login_attempts)
        .bind(user.locked_until)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_user_successful_login(&self, user: &AuthUser) -> ServiceResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_users
            SET failed_login_attempts = $1, locked_until = $2, last_login = $3, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(user.failed_login_attempts)
        .bind(user.locked_until)
        .bind(user.last_login)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_user_verification(&self, user: &AuthUser) -> ServiceResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_users
            SET is_verified = $1, verification_token = $2, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(user.is_verified)
        .bind(&user.verification_token)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_user_reset_token(&self, user: &AuthUser) -> ServiceResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_users
            SET reset_token = $1, reset_token_expires = $2, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(&user.reset_token)
        .bind(user.reset_token_expires)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_user_password(&self, user: &AuthUser) -> ServiceResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_users
            SET password_hash = $1, reset_token = $2, reset_token_expires = $3, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(&user.password_hash)
        .bind(&user.reset_token)
        .bind(user.reset_token_expires)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn create_test_pool() -> Pool<Postgres> {
        // This would need a test database URL
        // For now, we'll skip actual database tests
        todo!("Implement test database setup")
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_user_registration() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);

        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "Test123!@#".to_string(),
        };

        let result = auth_service.register(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_user_login() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);

        // First register a user
        let register_request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "Test123!@#".to_string(),
        };
        auth_service.register(register_request).await.unwrap();

        // Then try to login
        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "Test123!@#".to_string(),
        };

        let result = auth_service.login(login_request).await;
        assert!(result.is_ok());
    }
}
