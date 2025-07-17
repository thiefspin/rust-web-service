# Technical Specification: Building Rust Web Services with Actix v4

## Overview

This document outlines best practices and technical guidelines for building robust, maintainable, and performant web services in Rust using the Actix v4 framework. It is intended for AI agents and developers automating or contributing to Rust web service projects, ensuring consistency, reliability, and scalability.

## Table of Contents

1. Project Structure
2. Dependency Management
3. Application Initialization
4. Routing and Handlers
5. Request and Response Models
6. Error Handling
7. Middleware
8. State Management
9. Testing
10. Logging and Observability
11. Security Best Practices
12. Performance Considerations
13. Deployment Guidelines
14. Code Quality and Style

---

## 1. Project Structure

Organize the codebase for clarity and scalability:

```
rust-web-service/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── routes/
│   │   └── mod.rs
│   ├── handlers/
│   │   └── mod.rs
│   ├── models/
│   │   └── mod.rs
│   ├── middleware/
│   │   └── mod.rs
│   └── errors.rs
├── spec/
│   └── TECHNICAL_SPEC.md
├── Cargo.toml
└── README.md
```

- **src/main.rs**: Application entry point.
- **src/config.rs**: Configuration management.
- **src/routes/**: Route definitions.
- **src/handlers/**: Business logic for endpoints.
- **src/models/**: Data models (request/response structs).
- **src/middleware/**: Custom middleware.
- **src/errors.rs**: Error types and error handling utilities.

## 2. Dependency Management

- Use [Cargo.toml](../Cargo.toml) to declare dependencies.
- Pin versions for reproducibility (e.g., `actix-web = "4"`).
- Regularly audit dependencies for vulnerabilities (`cargo audit`).

## 3. Application Initialization

- Use `HttpServer::new` to initialize the Actix application.
- Configure routes, middleware, and shared state during initialization.
- Example:

```rust
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // .wrap(middleware)
            // .configure(routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## 4. Routing and Handlers

- Define routes in a dedicated module (`src/routes/`).
- Use RESTful conventions for endpoints.
- Handlers should be async functions, returning `impl Responder`.
- Example:

```rust
use actix_web::{get, web, HttpResponse, Responder};

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
```

## 5. Request and Response Models

- Define request and response structs in `src/models/`.
- Use `serde::{Serialize, Deserialize}` for JSON (de)serialization.
- Validate incoming data using crates like `validator` if needed.

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: u32,
    pub username: String,
    pub email: String,
}
```

## 6. Error Handling

- Centralize error types in `src/errors.rs`.
- Implement `ResponseError` for custom error types to control HTTP responses.
- Return meaningful error messages and status codes.

```rust
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Not found")]
    NotFound,
    #[error("Internal server error")]
    InternalError,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::NotFound => HttpResponse::NotFound().body(self.to_string()),
            ServiceError::InternalError => HttpResponse::InternalServerError().body(self.to_string()),
        }
    }
}
```

## 7. Middleware

- Use built-in middleware for logging, compression, CORS, etc.
- Implement custom middleware in `src/middleware/` for authentication, rate limiting, etc.
- Register middleware in the application initialization.

## 8. State Management

- Use `web::Data<T>` for shared application state (e.g., database pools, configuration).
- Ensure state is thread-safe (`Arc`, `Mutex`, etc. as needed).

```rust
use actix_web::web;

struct AppState {
    db_pool: DbPool,
}

App::new()
    .app_data(web::Data::new(AppState { db_pool }))
```

## 9. Testing

- Use `actix_web::test` utilities for endpoint testing.
- Write unit tests for handlers and integration tests for routes.
- Example test:

```rust
#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(App::new().service(health_check)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp
    }
}
```

## 10. PostgreSQL Integration with sqlx

This section covers best practices for integrating PostgreSQL using the `sqlx` crate.

### 10.1. Dependency Management

- Add `sqlx` and `sqlx-postgres` to `Cargo.toml`:
  ```
  sqlx = { version = "0.7", features = ["runtime-actix-native-tls", "postgres"] }
  ```
- Use the `runtime-actix-native-tls` feature for compatibility with Actix.

### 10.2. Database Connection

- Use a connection pool for efficient resource management:
  ```rust
  use sqlx::postgres::PgPoolOptions;

  async fn create_pool(database_url: &str) -> sqlx::Pool<sqlx::Postgres> {
      PgPoolOptions::new()
          .max_connections(5)
          .connect(database_url)
          .await
          .expect("Failed to create pool")
  }
  ```
- Store the pool in shared state (`web::Data`).

### 10.3. Migrations

- Use `sqlx-cli` for migrations:
  ```
  cargo install sqlx-cli
  sqlx migrate add <migration_name>
  sqlx migrate run
  ```
- Store migration files in a `migrations/` directory at the project root.

### 10.4. Querying

- Prefer compile-time checked queries with the `query!` macro:
  ```rust
  let row = sqlx::query!("SELECT id, username FROM users WHERE id = $1", user_id)
      .fetch_one(&pool)
      .await?;
  ```
- For dynamic queries, use `query` and map results manually.

### 10.5. Transactions

- Use transactions for multi-step operations:
  ```rust
  let mut tx = pool.begin().await?;
  // ... perform queries ...
  tx.commit().await?;
  ```

### 10.6. Error Handling

- Map `sqlx::Error` to your service error types.
- Return appropriate HTTP status codes for database errors.

### 10.7. Testing

- Use a test database or an in-memory database for integration tests.
- Clean up test data between runs.

### 10.8. Best Practices

- Never hardcode credentials; use environment variables or configuration files.
- Use connection pooling for performance.
- Validate and sanitize all inputs to queries.
- Handle database errors gracefully and log them for observability.
- Use migrations to manage schema changes.
