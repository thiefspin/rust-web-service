# Rust Web Service - Authentication System

A production-ready authentication system built with Rust and Actix-web v4, featuring JWT authentication, PostgreSQL integration, and comprehensive security features.

## Features

- 🔐 **JWT Authentication** - Secure token-based authentication
- 👤 **User Management** - Registration, login, email verification
- 🔒 **Password Security** - Bcrypt hashing with configurable cost
- 🛡️ **Account Protection** - Account locking after failed attempts
- 🔄 **Password Reset** - Secure password reset flow
- 📧 **Email Verification** - User email verification system
- 🚀 **High Performance** - Built with Actix-web for maximum performance
- 🗄️ **PostgreSQL Integration** - Robust database operations with sqlx
- ✅ **Input Validation** - Comprehensive request validation
- 📊 **Observability** - Structured logging and health checks
- 🧪 **Comprehensive Testing** - Unit and integration tests

## Architecture

The project follows the technical specification outlined in `spec/TECHNICAL_SPEC.md` with a clean, modular architecture:

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── errors.rs            # Error handling and types
├── routes/              # Route definitions
│   ├── mod.rs
│   └── auth_routes.rs
├── handlers/            # HTTP request handlers
│   ├── mod.rs
│   └── auth_handlers.rs
├── models/              # Data models and DTOs
│   ├── mod.rs
│   └── auth_user.rs
├── services/            # Business logic layer
│   ├── mod.rs
│   └── auth_service.rs
└── middleware/          # Custom middleware
    ├── mod.rs
    └── auth.rs
```

## Quick Start

### Prerequisites

- Rust 1.70+ 
- PostgreSQL 12+
- sqlx-cli for migrations

```bash
cargo install sqlx-cli
```

### Installation

1. **Clone the repository**
```bash
git clone <repository-url>
cd rust-web-service
```

2. **Set up environment variables**
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Set up the database**
```bash
# Create database
createdb rust_web_service

# Run migrations
sqlx migrate run
```

4. **Install dependencies and run**
```bash
cargo build
cargo run
```

The server will start on `http://127.0.0.1:8080` by default.

## Environment Configuration

Copy `.env.example` to `.env` and configure the following variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `JWT_SECRET` | JWT signing secret (min 32 chars) | Required |
| `JWT_EXPIRATION` | Token expiration in seconds | 3600 |
| `SERVER_HOST` | Server bind address | 127.0.0.1 |
| `SERVER_PORT` | Server port | 8080 |
| `BCRYPT_COST` | Bcrypt hashing cost (4-31) | 12 |
| `LOG_LEVEL` | Logging level | info |

## API Endpoints

### Public Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/register` | Register new user |
| POST | `/api/v1/auth/login` | User login |
| GET | `/api/v1/auth/verify-email` | Verify email address |
| POST | `/api/v1/auth/request-password-reset` | Request password reset |
| POST | `/api/v1/auth/confirm-password-reset` | Confirm password reset |
| GET | `/api/v1/health` | Health check |

### Protected Endpoints (Require JWT)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/auth/user/info` | Get user information |
| POST | `/api/v1/auth/user/change-password` | Change password |
| POST | `/api/v1/auth/user/refresh-token` | Refresh JWT token |
| POST | `/api/v1/auth/user/logout` | Logout user |

## Usage Examples

### User Registration

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!"
  }'
```

### User Login

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!"
  }'
```

Response:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "is_verified": false,
    "created_at": "2023-01-01T00:00:00Z",
    "last_login": "2023-01-01T12:00:00Z"
  }
}
```

### Accessing Protected Endpoints

```bash
curl -X GET http://localhost:8080/api/v1/auth/user/info \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Security Features

### Password Policy
- Minimum 8 characters
- Must contain uppercase, lowercase, digit, and special character
- Passwords are hashed using bcrypt with configurable cost

### Account Protection
- Account locking after 5 failed login attempts
- 15-minute lockout period
- Automatic unlock after lockout period

### JWT Security
- Configurable expiration time
- Secure secret key requirement (minimum 32 characters)
- Token validation on protected routes

### Request Validation
- Email format validation
- Password strength validation
- Input sanitization

## Database Schema

### auth_users Table

```sql
CREATE TABLE auth_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    verification_token VARCHAR(255),
    reset_token VARCHAR(255),
    reset_token_expires TIMESTAMPTZ
);
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test auth_service

# Run integration tests (requires test database)
cargo test --test integration_tests
```

## Development

### Code Style
The project follows Rust standard formatting:

```bash
# Format code
cargo fmt

# Check code style
cargo fmt --check

# Run linter
cargo clippy
```

### Database Migrations

Create a new migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
sqlx migrate run
```

Revert migrations:
```bash
sqlx migrate revert
```

## Production Deployment

### Docker Deployment

Create a `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-web-service /usr/local/bin/
EXPOSE 8080
CMD ["rust-web-service"]
```

### Environment Variables for Production

```bash
DATABASE_URL=postgresql://user:pass@db:5432/production_db
JWT_SECRET=your_production_secret_minimum_32_characters
JWT_EXPIRATION=3600
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
BCRYPT_COST=12
LOG_LEVEL=info
RUST_ENV=production
```

### Performance Considerations

- Connection pooling is configured with 5-20 database connections
- JWT tokens are stateless for horizontal scaling
- Bcrypt cost is configurable for performance tuning
- Request validation happens early in the pipeline

### Security Headers

The application includes security headers:
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
- Create an issue in the repository
- Check the technical specification in `spec/TECHNICAL_SPEC.md`
- Review the API documentation above

## Changelog

### v1.0.0
- Initial release with JWT authentication
- User registration and login
- Email verification system
- Password reset functionality
- Account security features
- Comprehensive testing suite