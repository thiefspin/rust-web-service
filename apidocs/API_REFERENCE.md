# Rust Web Service - Authentication API Reference

## Table of Contents

1. [Overview](#overview)
2. [Base URLs and Authentication](#base-urls-and-authentication)
3. [Response Format](#response-format)
4. [Error Handling](#error-handling)
5. [Public Endpoints](#public-endpoints)
6. [Protected Endpoints](#protected-endpoints)
7. [Authentication Flow Examples](#authentication-flow-examples)
8. [Testing with Different Scenarios](#testing-with-different-scenarios)
9. [Rate Limiting and Security](#rate-limiting-and-security)

## Overview

This API provides a complete authentication system with JWT-based authentication, user registration, login, password management, and account security features.

**API Version:** v1  
**Protocol:** HTTP/HTTPS  
**Authentication:** Bearer Token (JWT)  
**Content-Type:** application/json

## Base URLs and Authentication

### Base URL
```
http://127.0.0.1:8080/api/v1
```

### Authentication Header
For protected endpoints, include the JWT token in the Authorization header:
```
Authorization: Bearer <your_jwt_token>
```

## Response Format

All API responses follow a consistent JSON format:

### Success Response
```json
{
  "field1": "value1",
  "field2": "value2"
}
```

### Error Response
```json
{
  "error": "error_code",
  "message": "Human readable error message"
}
```

## Error Handling

### HTTP Status Codes

| Status Code | Description |
|-------------|-------------|
| 200 | OK - Request successful |
| 201 | Created - Resource created successfully |
| 400 | Bad Request - Invalid request data |
| 401 | Unauthorized - Authentication required or failed |
| 403 | Forbidden - Access denied |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists |
| 422 | Unprocessable Entity - Validation errors |
| 500 | Internal Server Error - Server error |

### Common Error Codes

| Error Code | Description |
|------------|-------------|
| `unauthorized` | Missing or invalid authentication |
| `forbidden` | Access denied |
| `validation_error` | Input validation failed |
| `user_already_exists` | User with email already exists |
| `invalid_credentials` | Wrong email or password |
| `token_expired` | JWT token has expired |
| `invalid_token` | JWT token is invalid |

---

## Public Endpoints

### 1. Health Check

Check if the API service is running and healthy.

**Endpoint:** `GET /health`

**Headers:** None required

**curl Example:**
```bash
curl -X GET "http://127.0.0.1:8080/api/v1/health"
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2023-12-07T10:30:00.000Z"
}
```

---

### 2. User Registration

Register a new user account.

**Endpoint:** `POST /auth/register`

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Password Requirements:**
- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one digit
- At least one special character (@$!%*?&)

**curl Example:**
```bash
curl -X POST "http://127.0.0.1:8080/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "password": "MySecurePass123!"
  }'
```

**Success Response (201 Created):**
```json
{
  "message": "User registered successfully. Please check your email for verification."
}
```

**Error Response (400 Bad Request):**
```json
{
  "error": "validation_error",
  "message": "email: Invalid email format; password: Password must be at least 8 characters long"
}
```

**Error Response (409 Conflict):**
```json
{
  "error": "user_already_exists",
  "message": "User already exists"
}
```

---

### 3. User Login

Authenticate a user and receive an access token.

**Endpoint:** `POST /auth/login`

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**curl Example:**
```bash
curl -X POST "http://127.0.0.1:8080/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "password": "MySecurePass123!"
  }'
```

**Success Response (200 OK):**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJlbWFpbCI6ImpvaG4uZG9lQGV4YW1wbGUuY29tIiwiZXhwIjoxNzAyMDE5NDAwLCJpYXQiOjE3MDIwMTU4MDB9.xyz123abc456def789ghi012jkl345mno678pqr901stu234vwx567yz8",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "john.doe@example.com",
    "is_verified": false,
    "created_at": "2023-12-07T10:00:00.000Z",
    "last_login": "2023-12-07T10:30:00.000Z"
  }
}
```

**Error Response (401 Unauthorized):**
```json
{
  "error": "invalid_credentials",
  "message": "Invalid credentials"
}
```

**Error Response (401 Unauthorized - Account Locked):**
```json
{
  "error": "unauthorized",
  "message": "Authentication failed"
}
```

---

### 4. Email Verification

Verify a user's email address using a verification token.

**Endpoint:** `GET /auth/verify-email`

**Headers:** None required

**Query Parameters:**
- `token` (required): Email verification token

**curl Example:**
```bash
curl -X GET "http://127.0.0.1:8080/api/v1/auth/verify-email?token=a1b2c3d4-e5f6-7890-abcd-ef1234567890"
```

**Success Response (200 OK):**
```json
{
  "message": "Email verified successfully."
}
```

**Error Response (401 Unauthorized):**
```json
{
  "error": "invalid_token",
  "message": "Invalid token"
}
```

---

### 5. Request Password Reset

Request a password reset link to be sent to the user's email.

**Endpoint:** `POST /auth/request-password-reset`

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{
  "email": "user@example.com"
}
```

**curl Example:**
```bash
curl -X POST "http://127.0.0.1:8080/api/v1/auth/request-password-reset" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com"
  }'
```

**Success Response (200 OK):**
```json
{
  "message": "If the email exists, a password reset link has been sent."
}
```

**Note:** For security reasons, this endpoint always returns the same message regardless of whether the email exists.

---

### 6. Confirm Password Reset

Reset a user's password using a reset token.

**Endpoint:** `POST /auth/confirm-password-reset`

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{
  "token": "reset-token-here",
  "new_password": "NewSecurePassword123!"
}
```

**curl Example:**
```bash
curl -X POST "http://127.0.0.1:8080/api/v1/auth/confirm-password-reset" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "b2c3d4e5-f6g7-8901-bcde-f23456789012",
    "new_password": "MyNewSecurePass123!"
  }'
```

**Success Response (200 OK):**
```json
{
  "message": "Password reset successfully."
}
```

**Error Response (401 Unauthorized):**
```json
{
  "error": "invalid_token",
  "message": "Invalid token"
}
```

---

## Protected Endpoints

All protected endpoints require a valid JWT token in the Authorization header.

### 7. Get User Info

Retrieve information about the currently authenticated user.

**Endpoint:** `GET /auth/user/info`

**Headers:**
- `Authorization: Bearer <jwt_token>`

**curl Example:**
```bash
# Save your token from login response
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

curl -X GET "http://127.0.0.1:8080/api/v1/auth/user/info" \
  -H "Authorization: Bearer $TOKEN"
```

**Success Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "john.doe@example.com",
  "is_verified": true,
  "created_at": "2023-12-07T10:00:00.000Z",
  "last_login": "2023-12-07T10:30:00.000Z"
}
```

**Error Response (401 Unauthorized):**
```json
{
  "error": "unauthorized",
  "message": "Authentication failed"
}
```

---

### 8. Change Password

Change the authenticated user's password.

**Endpoint:** `POST /auth/user/change-password`

**Headers:**
- `Authorization: Bearer <jwt_token>`
- `Content-Type: application/json`

**Request Body:**
```json
{
  "current_password": "CurrentPassword123!",
  "new_password": "NewSecurePassword123!"
}
```

**curl Example:**
```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

curl -X POST "http://127.0.0.1:8080/api/v1/auth/user/change-password" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "MySecurePass123!",
    "new_password": "MyNewSecurePass456!"
  }'
```

**Success Response (200 OK):**
```json
{
  "message": "Password changed successfully."
}
```

**Error Response (401 Unauthorized):**
```json
{
  "error": "invalid_credentials",
  "message": "Invalid credentials"
}
```

---

### 9. Refresh Token

Generate a new JWT token for the authenticated user.

**Endpoint:** `POST /auth/user/refresh-token`

**Headers:**
- `Authorization: Bearer <jwt_token>`

**curl Example:**
```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

curl -X POST "http://127.0.0.1:8080/api/v1/auth/user/refresh-token" \
  -H "Authorization: Bearer $TOKEN"
```

**Success Response (200 OK):**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.NEW_TOKEN_CONTENT",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "john.doe@example.com",
    "is_verified": true,
    "created_at": "2023-12-07T10:00:00.000Z",
    "last_login": "2023-12-07T10:30:00.000Z"
  }
}
```

---

### 10. Logout

Logout the current user (primarily for logging purposes).

**Endpoint:** `POST /auth/user/logout`

**Headers:**
- `Authorization: Bearer <jwt_token>`

**curl Example:**
```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

curl -X POST "http://127.0.0.1:8080/api/v1/auth/user/logout" \
  -H "Authorization: Bearer $TOKEN"
```

**Success Response (200 OK):**
```json
{
  "message": "Logged out successfully"
}
```

**Note:** With JWT tokens, logout is typically handled client-side by removing the token. This endpoint serves for logging purposes.

---

## Authentication Flow Examples

### Complete Registration and Login Flow

```bash
# 1. Register a new user
curl -X POST "http://127.0.0.1:8080/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "AliceSecure123!"
  }'

# 2. Login to get access token
RESPONSE=$(curl -X POST "http://127.0.0.1:8080/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "AliceSecure123!"
  }')

# 3. Extract token (using jq if available)
TOKEN=$(echo $RESPONSE | jq -r '.access_token')

# 4. Access protected endpoint
curl -X GET "http://127.0.0.1:8080/api/v1/auth/user/info" \
  -H "Authorization: Bearer $TOKEN"

# 5. Change password
curl -X POST "http://127.0.0.1:8080/api/v1/auth/user/change-password" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "AliceSecure123!",
    "new_password": "AliceNewSecure456!"
  }'

# 6. Logout
curl -X POST "http://127.0.0.1:8080/api/v1/auth/user/logout" \
  -H "Authorization: Bearer $TOKEN"
```

### Password Reset Flow

```bash
# 1. Request password reset
curl -X POST "http://127.0.0.1:8080/api/v1/auth/request-password-reset" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com"
  }'

# 2. Check server logs for reset token (in development)
# In production, this would be sent via email

# 3. Reset password with token
curl -X POST "http://127.0.0.1:8080/api/v1/auth/confirm-password-reset" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "RESET_TOKEN_FROM_EMAIL",
    "new_password": "AliceResetPass789!"
  }'

# 4. Login with new password
curl -X POST "http://127.0.0.1:8080/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "AliceResetPass789!"
  }'
```

---

## Testing with Different Scenarios

### Testing Validation Errors

```bash
# Invalid email format
curl -X POST "http://127.0.0.1:8080/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid-email",
    "password": "ValidPass123!"
  }'

# Weak password
curl -X POST "http://127.0.0.1:8080/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "weak"
  }'

# Missing fields
curl -X POST "http://127.0.0.1:8080/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com"
  }'
```

### Testing Account Security

```bash
# Test multiple failed login attempts (will lock account after 5 attempts)
for i in {1..6}; do
  echo "Attempt $i:"
  curl -X POST "http://127.0.0.1:8080/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{
      "email": "alice@example.com",
      "password": "WrongPassword"
    }'
  echo -e "\n"
done
```

### Testing Protected Endpoints Without Authentication

```bash
# Try to access protected endpoint without token
curl -X GET "http://127.0.0.1:8080/api/v1/auth/user/info"

# Try to access with invalid token
curl -X GET "http://127.0.0.1:8080/api/v1/auth/user/info" \
  -H "Authorization: Bearer invalid_token_here"

# Try to access with malformed Authorization header
curl -X GET "http://127.0.0.1:8080/api/v1/auth/user/info" \
  -H "Authorization: invalid_format"
```

---

## Rate Limiting and Security

### Security Headers

All responses include security headers:
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`

### Account Security Features

1. **Password Requirements:** Strong password policy enforced
2. **Account Locking:** Account locked for 15 minutes after 5 failed login attempts
3. **JWT Expiration:** Tokens expire after configured time (default: 1 hour)
4. **Password Reset Tokens:** Time-limited reset tokens (1 hour expiration)
5. **Email Verification:** Users must verify email addresses

### Best Practices for API Usage

1. **Store tokens securely:** Use secure storage mechanisms (not localStorage for web apps)
2. **Handle token expiration:** Implement token refresh logic
3. **Use HTTPS in production:** Never send tokens over unencrypted connections
4. **Validate responses:** Always check response status and handle errors
5. **Rate limiting:** Implement client-side rate limiting to avoid hitting server limits

### Development vs Production

**Development Environment:**
- Reset tokens are logged to console
- Detailed error messages
- CORS enabled for localhost

**Production Environment:**
- Reset tokens sent via email
- Generic error messages for security
- Strict CORS policy
- Rate limiting enabled
- HTTPS required

---

## Troubleshooting

### Common Issues and Solutions

1. **401 Unauthorized on protected endpoints**
   - Check if token is included in Authorization header
   - Verify token format: `Bearer <token>`
   - Check if token has expired

2. **400 Bad Request on registration/login**
   - Verify JSON format
   - Check password meets requirements
   - Ensure email format is valid

3. **409 Conflict on registration**
   - User with email already exists
   - Try login instead or use different email

4. **Connection errors**
   - Verify server is running on correct port
   - Check network connectivity
   - Verify base URL is correct

### Testing Tools

You can also test the API using:
- **Postman:** Import the curl commands as a collection
- **Insomnia:** REST client with good JSON support
- **HTTPie:** Command-line tool with simpler syntax than curl
- **Browser DevTools:** For debugging frontend integration

### Example with HTTPie

```bash
# Install HTTPie
pip install httpie

# Register user
http POST 127.0.0.1:8080/api/v1/auth/register email=test@example.com password=SecurePass123!

# Login
http POST 127.0.0.1:8080/api/v1/auth/login email=test@example.com password=SecurePass123!

# Access protected endpoint
http GET 127.0.0.1:8080/api/v1/auth/user/info Authorization:"Bearer YOUR_TOKEN"
```

This API reference provides all the information needed to integrate with the Rust Web Service authentication system. For more details about the implementation, see the main [README.md](../README.md) file.