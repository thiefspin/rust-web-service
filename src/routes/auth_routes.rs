use actix_web::web;

use crate::handlers::auth_handlers::{
    change_password, confirm_password_reset, get_user_info, health_check, login, logout,
    refresh_token, register, request_password_reset, verify_email,
};
use crate::middleware::auth::JwtAuth;

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/auth")
            // Public routes (no authentication required)
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/verify-email", web::get().to(verify_email))
            .route(
                "/request-password-reset",
                web::post().to(request_password_reset),
            )
            .route(
                "/confirm-password-reset",
                web::post().to(confirm_password_reset),
            )
            .route("/health", web::get().to(health_check))
            // Protected routes (authentication required)
            .service(
                web::scope("/user")
                    .wrap(JwtAuth)
                    .route("/info", web::get().to(get_user_info))
                    .route("/change-password", web::post().to(change_password))
                    .route("/refresh-token", web::post().to(refresh_token))
                    .route("/logout", web::post().to(logout)),
            ),
    );
}

pub fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").route("/health", web::get().to(health_check)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_route_configuration() {
        let app = test::init_service(
            App::new()
                .configure(configure_auth_routes)
                .configure(configure_public_routes),
        )
        .await;

        // Test that the routes are properly configured
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/health")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_protected_routes_require_auth() {
        let app = test::init_service(App::new().configure(configure_auth_routes)).await;

        // Test that protected routes return 401 without authentication
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/user/info")
            .to_request();
        let resp = test::try_call_service(&app, req).await;
        assert!(resp.is_err());

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/user/logout")
            .to_request();
        let resp = test::try_call_service(&app, req).await;
        assert!(resp.is_err());
    }

    #[actix_web::test]
    async fn test_public_routes_no_auth() {
        let app = test::init_service(App::new().configure(configure_auth_routes)).await;

        // Test that public routes work without authentication
        let public_routes = vec![
            "/api/v1/auth/health",
            // Note: POST routes would need proper request bodies to test fully
        ];

        for route in public_routes {
            let req = test::TestRequest::get().uri(route).to_request();
            let resp = test::call_service(&app, req).await;
            // Should not be 401 (unauthorized)
            assert_ne!(resp.status(), 401);
        }
    }
}
