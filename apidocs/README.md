# API Documentation

This directory contains comprehensive documentation for the Rust Web Service Authentication API.

## 📁 Files Overview

| File | Description | Format |
|------|-------------|---------|
| `API_REFERENCE.md` | Complete API documentation with curl examples | Markdown |
| `QUICK_START.md` | 5-minute guide to get started with the API | Markdown |
| `openapi.yaml` | OpenAPI 3.0 specification for API tools | YAML |
| `postman_collection.json` | Postman collection for testing | JSON |

## 🚀 Quick Start

1. **Read the Quick Start Guide**: [`QUICK_START.md`](./QUICK_START.md)
2. **Try the API**: Follow the 5-minute tutorial
3. **Import to Postman**: Use [`postman_collection.json`](./postman_collection.json)
4. **View in Swagger**: Load [`openapi.yaml`](./openapi.yaml) in Swagger UI

## 📖 Documentation Types

### API Reference
**File**: [`API_REFERENCE.md`](./API_REFERENCE.md)

Complete documentation including:
- All endpoints with detailed descriptions
- curl command examples for every endpoint
- Request/response schemas
- Error codes and handling
- Authentication flows
- Testing scenarios

### Quick Start Guide
**File**: [`QUICK_START.md`](./QUICK_START.md)

Get up and running in 5 minutes:
- Health check verification
- User registration and login
- Token usage examples
- Password management
- Complete workflow scripts

### OpenAPI Specification
**File**: [`openapi.yaml`](./openapi.yaml)

Machine-readable API specification:
- OpenAPI 3.0 format
- Complete schema definitions
- Can be imported into Swagger UI, Postman, or other tools
- Supports code generation

### Postman Collection
**File**: [`postman_collection.json`](./postman_collection.json)

Ready-to-use Postman collection:
- All endpoints configured
- Environment variables set up
- Test scripts included
- Error scenario testing

## 🛠️ Using the Documentation

### With curl (Command Line)
```bash
# Follow examples in API_REFERENCE.md or QUICK_START.md
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!"}'
```

### With Postman
1. Import `postman_collection.json` into Postman
2. Set up environment variables:
   - `base_url`: `http://127.0.0.1:8080/api/v1`
   - `test_email`: Your test email
   - `test_password`: Your test password
3. Run the collection or individual requests

### With Swagger UI
1. Go to [Swagger Editor](https://editor.swagger.io/)
2. Copy content from `openapi.yaml`
3. Paste into the editor
4. Explore and test the API interactively

### With Other Tools
- **Insomnia**: Import the OpenAPI spec
- **HTTPie**: Use the curl examples with HTTPie syntax
- **Bruno**: Import the OpenAPI specification
- **VSCode REST Client**: Copy curl examples to `.http` files

## 🔗 API Endpoints Summary

### Public Endpoints (No Auth Required)
- `GET /health` - Health check
- `POST /auth/register` - User registration
- `POST /auth/login` - User login
- `GET /auth/verify-email` - Email verification
- `POST /auth/request-password-reset` - Request password reset
- `POST /auth/confirm-password-reset` - Confirm password reset

### Protected Endpoints (JWT Required)
- `GET /auth/user/info` - Get user information
- `POST /auth/user/change-password` - Change password
- `POST /auth/user/refresh-token` - Refresh JWT token
- `POST /auth/user/logout` - Logout user

## 🔐 Authentication

All protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

Get a token by logging in:
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"your@email.com","password":"YourPassword123!"}'
```

## ⚡ Environment Setup

### Development Server
```bash
# Default development URL
BASE_URL=http://127.0.0.1:8080/api/v1
```

### Production Server
```bash
# Replace with your production URL
BASE_URL=https://api.yourapp.com/api/v1
```

## 🧪 Testing

### Automated Testing
```bash
# Run the provided integration test script
cd ../
./test_auth.sh
```

### Manual Testing
1. Use the examples in `QUICK_START.md`
2. Import and run the Postman collection
3. Try the error scenarios in `API_REFERENCE.md`

## 📝 Error Handling

Common HTTP status codes:
- `200` - Success
- `201` - Created
- `400` - Bad Request (validation errors)
- `401` - Unauthorized (authentication required)
- `403` - Forbidden (access denied)
- `404` - Not Found
- `409` - Conflict (user already exists)
- `500` - Internal Server Error

See [`API_REFERENCE.md`](./API_REFERENCE.md) for detailed error responses.

## 🔧 Troubleshooting

### Server Not Responding
```bash
# Check if server is running
curl http://127.0.0.1:8080/api/v1/health

# Start the server if needed
cd ../
cargo run
```

### Authentication Issues
- Check token format: `Bearer <token>`
- Verify token hasn't expired (default: 1 hour)
- Ensure you're using the correct endpoint URLs

### Validation Errors
- Email must be valid format
- Password must meet requirements (8+ chars, uppercase, lowercase, digit, special)
- Check request body JSON format

## 📚 Additional Resources

- **Main README**: [`../README.md`](../README.md) - Setup and deployment
- **Technical Spec**: [`../spec/TECHNICAL_SPEC.md`](../spec/TECHNICAL_SPEC.md) - Implementation details
- **Source Code**: [`../src/`](../src/) - Implementation
- **Tests**: [`../test_auth.sh`](../test_auth.sh) - Integration tests

## 🤝 Contributing

When updating the API:
1. Update the OpenAPI spec in `openapi.yaml`
2. Update examples in `API_REFERENCE.md`
3. Add new endpoints to the Postman collection
4. Update this README if needed

## 📞 Support

For API issues:
- Check the troubleshooting section above
- Review error messages in the API responses
- Test with the provided examples
- Check server logs for additional context

---

**Happy coding! 🎉**