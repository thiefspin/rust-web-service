use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use uuid::Uuid;

use crate::{config::CONFIG, errors::ServiceError, models::auth_user::Claims};

// Middleware factory
pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

// Middleware service
pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[allow(unused_mut)]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract the Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(auth_header) = auth_header {
                if let Some(token) = auth_header.strip_prefix("Bearer ") {
                    match verify_jwt_token(token) {
                        Ok(claims) => {
                            // Add user info to request extensions
                            req.extensions_mut().insert(AuthenticatedUser {
                                user_id: claims
                                    .sub
                                    .parse()
                                    .map_err(|_| ServiceError::InvalidToken)?,
                                email: claims.email,
                            });
                        }
                        Err(e) => {
                            log::warn!("JWT verification failed: {e:?}");
                            return Err(actix_web::error::ErrorUnauthorized(e));
                        }
                    }
                } else {
                    return Err(actix_web::error::ErrorUnauthorized(
                        ServiceError::InvalidToken,
                    ));
                }
            } else {
                return Err(actix_web::error::ErrorUnauthorized(
                    ServiceError::Unauthorized,
                ));
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

// Optional middleware for routes that can work with or without authentication
pub struct OptionalJwtAuth;

impl<S, B> Transform<S, ServiceRequest> for OptionalJwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OptionalJwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OptionalJwtAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct OptionalJwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for OptionalJwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[allow(unused_mut)]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Try to extract and verify JWT token, but don't fail if it's missing
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(auth_header) = auth_header {
                if let Some(token) = auth_header.strip_prefix("Bearer ") {
                    if let Ok(claims) = verify_jwt_token(token) {
                        if let Ok(user_id) = claims.sub.parse() {
                            req.extensions_mut().insert(AuthenticatedUser {
                                user_id,
                                email: claims.email,
                            });
                        }
                    }
                }
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

// User information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

// JWT token verification function
pub fn verify_jwt_token(token: &str) -> Result<Claims, ServiceError> {
    let decoding_key = DecodingKey::from_secret(CONFIG.jwt_secret_bytes());
    let validation = Validation::default();

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(ServiceError::from)
}

// JWT token generation function
pub fn generate_jwt_token(user_id: Uuid, email: String) -> Result<String, ServiceError> {
    let claims = Claims::new(user_id, email, CONFIG.jwt_expiration);
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(CONFIG.jwt_secret_bytes());

    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key)
        .map_err(ServiceError::from)
}

// Helper trait to extract authenticated user from request
pub trait AuthenticatedUserExt {
    fn authenticated_user(&self) -> Option<AuthenticatedUser>;
    fn require_authenticated_user(&self) -> Result<AuthenticatedUser, ServiceError>;
}

impl AuthenticatedUserExt for ServiceRequest {
    fn authenticated_user(&self) -> Option<AuthenticatedUser> {
        self.extensions().get::<AuthenticatedUser>().cloned()
    }

    fn require_authenticated_user(&self) -> Result<AuthenticatedUser, ServiceError> {
        self.authenticated_user().ok_or(ServiceError::Unauthorized)
    }
}

impl AuthenticatedUserExt for actix_web::HttpRequest {
    fn authenticated_user(&self) -> Option<AuthenticatedUser> {
        self.extensions().get::<AuthenticatedUser>().cloned()
    }

    fn require_authenticated_user(&self) -> Result<AuthenticatedUser, ServiceError> {
        self.authenticated_user().ok_or(ServiceError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn protected_handler(req: actix_web::HttpRequest) -> Result<HttpResponse, ServiceError> {
        let user = req.require_authenticated_user()?;
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "user_id": user.user_id,
            "email": user.email
        })))
    }

    #[actix_web::test]
    async fn test_jwt_auth_middleware_without_token() {
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth)
                .route("/protected", web::get().to(protected_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/protected").to_request();
        let resp = test::try_call_service(&app, req).await;

        assert!(resp.is_err());
    }

    #[actix_web::test]
    async fn test_jwt_auth_middleware_with_invalid_token() {
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth)
                .route("/protected", web::get().to(protected_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer invalid_token"))
            .to_request();
        let resp = test::try_call_service(&app, req).await;

        assert!(resp.is_err());
    }

    #[actix_web::test]
    async fn test_optional_jwt_auth_middleware_without_token() {
        async fn optional_handler(req: actix_web::HttpRequest) -> HttpResponse {
            if let Some(user) = req.authenticated_user() {
                HttpResponse::Ok().json(serde_json::json!({
                    "authenticated": true,
                    "user_id": user.user_id,
                    "email": user.email
                }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "authenticated": false
                }))
            }
        }

        let app = test::init_service(
            App::new()
                .wrap(OptionalJwtAuth)
                .route("/optional", web::get().to(optional_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/optional").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
    }
}
