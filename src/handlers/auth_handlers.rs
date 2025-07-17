use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_validator::{Json, Query};
use validator::Validate;

use crate::{
    errors::ServiceResult,
    middleware::auth::AuthenticatedUserExt,
    models::auth_user::{
        ChangePasswordRequest, ConfirmResetPasswordRequest, LoginRequest, RegisterRequest,
        ResetPasswordRequest, VerifyEmailRequest,
    },
    services::AuthService,
};

/// Register a new user
pub async fn register(
    auth_service: web::Data<AuthService>,
    Json(request): Json<RegisterRequest>,
) -> ServiceResult<impl Responder> {
    // Validate the request
    request.validate()?;

    let response = auth_service.register(request).await?;
    Ok(HttpResponse::Created().json(response))
}

/// Login a user
pub async fn login(
    auth_service: web::Data<AuthService>,
    Json(request): Json<LoginRequest>,
) -> ServiceResult<impl Responder> {
    // Validate the request
    request.validate()?;

    let response = auth_service.login(request).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Verify email address
pub async fn verify_email(
    auth_service: web::Data<AuthService>,
    Query(request): Query<VerifyEmailRequest>,
) -> ServiceResult<impl Responder> {
    let response = auth_service.verify_email(request).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Request password reset
pub async fn request_password_reset(
    auth_service: web::Data<AuthService>,
    Json(request): Json<ResetPasswordRequest>,
) -> ServiceResult<impl Responder> {
    // Validate the request
    request.validate()?;

    let response = auth_service.request_password_reset(request).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Confirm password reset
pub async fn confirm_password_reset(
    auth_service: web::Data<AuthService>,
    Json(request): Json<ConfirmResetPasswordRequest>,
) -> ServiceResult<impl Responder> {
    // Validate the request
    request.validate()?;

    let response = auth_service.confirm_password_reset(request).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Change password (requires authentication)
pub async fn change_password(
    req: HttpRequest,
    auth_service: web::Data<AuthService>,
    Json(request): Json<ChangePasswordRequest>,
) -> ServiceResult<impl Responder> {
    // Validate the request
    request.validate()?;

    let user = req.require_authenticated_user()?;
    let response = auth_service.change_password(user.user_id, request).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Get current user info (requires authentication)
pub async fn get_user_info(
    req: HttpRequest,
    auth_service: web::Data<AuthService>,
) -> ServiceResult<impl Responder> {
    let user = req.require_authenticated_user()?;
    let response = auth_service.get_user_info(user.user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Refresh JWT token (requires authentication)
pub async fn refresh_token(
    req: HttpRequest,
    auth_service: web::Data<AuthService>,
) -> ServiceResult<impl Responder> {
    let user = req.require_authenticated_user()?;
    let response = auth_service.refresh_token(user.user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Health check endpoint (no authentication required)
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Logout endpoint (requires authentication)
/// Note: With JWT, logout is typically handled client-side by removing the token
/// This endpoint can be used for logging purposes or token blacklisting
pub async fn logout(req: HttpRequest) -> ServiceResult<impl Responder> {
    let user = req.require_authenticated_user()?;

    // Log the logout event
    log::info!("User {} logged out", user.email);

    // In a more sophisticated implementation, you might:
    // - Add the token to a blacklist
    // - Store logout events in the database
    // - Send notifications

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::models::auth_user::MessageResponse; // Commented out due to unused import
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app =
            test::init_service(App::new().route("/health", web::get().to(health_check))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_register_request_validation() {
        let invalid_request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "weak".to_string(),
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_login_request_validation() {
        let invalid_request = LoginRequest {
            email: "invalid-email".to_string(),
            password: "".to_string(),
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_change_password_request_validation() {
        let invalid_request = ChangePasswordRequest {
            current_password: "".to_string(),
            new_password: "weak".to_string(),
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_reset_password_request_validation() {
        let invalid_request = ResetPasswordRequest {
            email: "invalid-email".to_string(),
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
    }

    #[tokio::test]
    async fn test_confirm_reset_password_request_validation() {
        let invalid_request = ConfirmResetPasswordRequest {
            token: "some-token".to_string(),
            new_password: "weak".to_string(),
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());
    }
}
