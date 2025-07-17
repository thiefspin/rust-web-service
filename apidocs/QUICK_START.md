# Quick Start Guide - Authentication API

This guide will get you up and running with the authentication API in 5 minutes.

## Prerequisites

- Server running on `http://127.0.0.1:8080`
- `curl` command available
- `jq` (optional, for JSON parsing)

## 1. Health Check

First, verify the API is running:

```bash
curl http://127.0.0.1:8080/api/v1/health
```

Expected response:
```json
{"status":"healthy","timestamp":"2023-12-07T10:30:00.000Z"}
```

## 2. Register a New User

```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "quickstart@example.com",
    "password": "QuickStart123!"
  }'
```

Expected response:
```json
{"message":"User registered successfully. Please check your email for verification."}
```

## 3. Login and Get Token

```bash
# Save the response to extract the token
RESPONSE=$(curl -s -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "quickstart@example.com",
    "password": "QuickStart123!"
  }')

# Extract token (with jq)
TOKEN=$(echo $RESPONSE | jq -r '.access_token')

# Or extract token manually (without jq)
# TOKEN="paste_token_here_from_response"

echo "Token: $TOKEN"
```

## 4. Access Protected Endpoint

```bash
curl -X GET http://127.0.0.1:8080/api/v1/auth/user/info \
  -H "Authorization: Bearer $TOKEN"
```

Expected response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "quickstart@example.com",
  "is_verified": false,
  "created_at": "2023-12-07T10:00:00.000Z",
  "last_login": "2023-12-07T10:30:00.000Z"
}
```

## 5. Change Password

```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/user/change-password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "QuickStart123!",
    "new_password": "NewQuickStart456!"
  }'
```

## 6. Test with New Password

```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "quickstart@example.com",
    "password": "NewQuickStart456!"
  }'
```

## Complete Script

Here's a complete script you can run:

```bash
#!/bin/bash

BASE_URL="http://127.0.0.1:8080/api/v1"
EMAIL="quickstart@example.com"
ORIGINAL_PASSWORD="QuickStart123!"
NEW_PASSWORD="NewQuickStart456!"

echo "🔍 Testing health endpoint..."
curl -s $BASE_URL/health | jq '.'

echo -e "\n👤 Registering user..."
curl -s -X POST $BASE_URL/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$ORIGINAL_PASSWORD\"}" | jq '.'

echo -e "\n🔑 Logging in..."
RESPONSE=$(curl -s -X POST $BASE_URL/auth/login \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$ORIGINAL_PASSWORD\"}")

TOKEN=$(echo $RESPONSE | jq -r '.access_token')
echo "Token received: ${TOKEN:0:50}..."

echo -e "\n📋 Getting user info..."
curl -s -X GET $BASE_URL/auth/user/info \
  -H "Authorization: Bearer $TOKEN" | jq '.'

echo -e "\n🔒 Changing password..."
curl -s -X POST $BASE_URL/auth/user/change-password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"current_password\":\"$ORIGINAL_PASSWORD\",\"new_password\":\"$NEW_PASSWORD\"}" | jq '.'

echo -e "\n🔑 Testing login with new password..."
curl -s -X POST $BASE_URL/auth/login \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$NEW_PASSWORD\"}" | jq '.'

echo -e "\n✅ Quick start complete!"
```

## Error Testing

Try these to see error responses:

```bash
# Invalid email
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid-email","password":"Test123!"}'

# Weak password
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"weak"}'

# Access protected endpoint without token
curl http://127.0.0.1:8080/api/v1/auth/user/info
```

## Next Steps

- Check the full [API Reference](./API_REFERENCE.md) for complete documentation
- Use the [test script](../test_auth.sh) for comprehensive testing
- Read the [README](../README.md) for deployment and development information

## Troubleshooting

**Server not responding?**
```bash
# Check if server is running
ps aux | grep rust-web-service

# Start the server
cd rust-web-service
cargo run
```

**JSON parsing errors?**
```bash
# Install jq for better JSON handling
brew install jq  # macOS
sudo apt install jq  # Ubuntu
```

**Token extraction without jq?**
```bash
# Manual token extraction
RESPONSE='{"access_token":"your_token_here",...}'
TOKEN="paste_actual_token_here"
```
