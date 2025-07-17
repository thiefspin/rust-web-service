use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use sqlx::postgres::PgPoolOptions;

mod config;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use config::CONFIG;
use routes::{configure_auth_routes, configure_public_routes};
use services::AuthService;

// Application state
pub struct AppState {
    auth_service: AuthService,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or(&CONFIG.log_level)).init();

    // Validate configuration
    if let Err(e) = CONFIG.validate() {
        log::error!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    log::info!("Starting Rust Web Service");
    log::info!("Configuration loaded successfully");

    // Create database connection pool
    log::info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(&CONFIG.database_url)
        .await
        .expect("Failed to connect to database");

    log::info!("Database connected successfully");

    // Run migrations
    log::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Database migrations completed");

    // Create services
    let auth_service = AuthService::new(db_pool.clone());

    // Create application state
    let app_state = AppState { auth_service };

    log::info!("Starting HTTP server on {}", CONFIG.server_address());

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Add application state
            .app_data(web::Data::new(app_state.auth_service.clone()))
            // Add middleware
            .wrap(Logger::default())
            .wrap(
                actix_web::middleware::DefaultHeaders::new()
                    .add(("X-Version", "1.0"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-XSS-Protection", "1; mode=block"))
                    .add((
                        "Strict-Transport-Security",
                        "max-age=31536000; includeSubDomains",
                    )),
            )
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin("http://localhost:3000") // Adjust for your frontend
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec!["Authorization", "Content-Type", "X-Requested-With"])
                    .supports_credentials()
                    .max_age(3600),
            )
            // Configure routes
            .configure(configure_auth_routes)
            .configure(configure_public_routes)
            // Add a catch-all route for unmatched requests
            .default_service(web::route().to(|| async {
                actix_web::HttpResponse::NotFound().json(serde_json::json!({
                    "error": "not_found",
                    "message": "The requested resource was not found"
                }))
            }))
    })
    .bind(&CONFIG.server_address())?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_app_configuration() {
        // Set test environment variables
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
        env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");
        env::set_var("LOG_LEVEL", "debug");

        // This test just ensures the app can be configured without panicking
        // In a real test environment, you'd need a test database
        let app = test::init_service(
            App::new()
                .configure(configure_auth_routes)
                .configure(configure_public_routes),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[test]
    fn test_config_validation() {
        // Test that config validation works
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
        env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");

        let config = config::Config::from_env().unwrap();
        assert!(config.validate().is_ok());
    }
}
