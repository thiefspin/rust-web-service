#!/bin/bash

# Integration test script for Rust Web Service Authentication
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="http://127.0.0.1:8080"
API_BASE="$BASE_URL/api/v1"
AUTH_BASE="$API_BASE/auth"

# Test user data
TEST_EMAIL="test@example.com"
TEST_PASSWORD="TestPassword123!"
TEST_NEW_PASSWORD="NewPassword123!"

# Function to print colored output
print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Function to make HTTP requests with error handling
make_request() {
    local method=$1
    local url=$2
    local data=$3
    local headers=$4

    if [ -n "$headers" ]; then
        curl -s -X "$method" "$url" \
             -H "Content-Type: application/json" \
             -H "$headers" \
             -d "$data"
    else
        curl -s -X "$method" "$url" \
             -H "Content-Type: application/json" \
             -d "$data"
    fi
}

# Function to extract JSON field
extract_json_field() {
    local json=$1
    local field=$2
    echo "$json" | grep -o "\"$field\":\"[^\"]*\"" | cut -d'"' -f4
}

# Function to check if server is running
check_server() {
    print_info "Checking if server is running..."
    if curl -s "$API_BASE/health" > /dev/null 2>&1; then
        print_success "Server is running"
        return 0
    else
        print_error "Server is not running. Please start the server first."
        return 1
    fi
}

# Test health endpoint
test_health_endpoint() {
    print_test "Testing health endpoint"

    response=$(curl -s "$API_BASE/health")

    if echo "$response" | grep -q "healthy"; then
        print_success "Health endpoint working"
    else
        print_error "Health endpoint failed"
        echo "Response: $response"
        return 1
    fi
}

# Test user registration
test_user_registration() {
    print_test "Testing user registration"

    local json_data="{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}"
    local response=$(make_request "POST" "$AUTH_BASE/register" "$json_data")

    if echo "$response" | grep -q "registered successfully"; then
        print_success "User registration successful"
    else
        print_error "User registration failed"
        echo "Response: $response"
        return 1
    fi
}

# Test user login
test_user_login() {
    print_test "Testing user login"

    local json_data="{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}"
    local response=$(make_request "POST" "$AUTH_BASE/login" "$json_data")

    if echo "$response" | grep -q "access_token"; then
        ACCESS_TOKEN=$(extract_json_field "$response" "access_token")
        print_success "User login successful"
        print_info "Access token obtained: ${ACCESS_TOKEN:0:20}..."
    else
        print_error "User login failed"
        echo "Response: $response"
        return 1
    fi
}

# Test accessing protected endpoint
test_protected_endpoint() {
    print_test "Testing protected endpoint access"

    if [ -z "$ACCESS_TOKEN" ]; then
        print_error "No access token available"
        return 1
    fi

    local response=$(make_request "GET" "$AUTH_BASE/user/info" "" "Authorization: Bearer $ACCESS_TOKEN")

    if echo "$response" | grep -q "$TEST_EMAIL"; then
        print_success "Protected endpoint access successful"
    else
        print_error "Protected endpoint access failed"
        echo "Response: $response"
        return 1
    fi
}

# Test accessing protected endpoint without token
test_unauthorized_access() {
    print_test "Testing unauthorized access to protected endpoint"

    local response=$(curl -s "$AUTH_BASE/user/info")

    if echo "$response" | grep -q "unauthorized"; then
        print_success "Unauthorized access properly blocked"
    else
        print_error "Unauthorized access not properly blocked"
        echo "Response: $response"
        return 1
    fi
}

# Test password change
test_password_change() {
    print_test "Testing password change"

    if [ -z "$ACCESS_TOKEN" ]; then
        print_error "No access token available"
        return 1
    fi

    local json_data="{\"current_password\":\"$TEST_PASSWORD\",\"new_password\":\"$TEST_NEW_PASSWORD\"}"
    local response=$(make_request "POST" "$AUTH_BASE/user/change-password" "$json_data" "Authorization: Bearer $ACCESS_TOKEN")

    if echo "$response" | grep -q "changed successfully"; then
        print_success "Password change successful"
        # Update password for subsequent tests
        TEST_PASSWORD="$TEST_NEW_PASSWORD"
    else
        print_error "Password change failed"
        echo "Response: $response"
        return 1
    fi
}

# Test login with new password
test_login_new_password() {
    print_test "Testing login with new password"

    local json_data="{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}"
    local response=$(make_request "POST" "$AUTH_BASE/login" "$json_data")

    if echo "$response" | grep -q "access_token"; then
        ACCESS_TOKEN=$(extract_json_field "$response" "access_token")
        print_success "Login with new password successful"
    else
        print_error "Login with new password failed"
        echo "Response: $response"
        return 1
    fi
}

# Test token refresh
test_token_refresh() {
    print_test "Testing token refresh"

    if [ -z "$ACCESS_TOKEN" ]; then
        print_error "No access token available"
        return 1
    fi

    local response=$(make_request "POST" "$AUTH_BASE/user/refresh-token" "" "Authorization: Bearer $ACCESS_TOKEN")

    if echo "$response" | grep -q "access_token"; then
        local new_token=$(extract_json_field "$response" "access_token")
        print_success "Token refresh successful"
        print_info "New token: ${new_token:0:20}..."
        ACCESS_TOKEN="$new_token"
    else
        print_error "Token refresh failed"
        echo "Response: $response"
        return 1
    fi
}

# Test password reset request
test_password_reset_request() {
    print_test "Testing password reset request"

    local json_data="{\"email\":\"$TEST_EMAIL\"}"
    local response=$(make_request "POST" "$AUTH_BASE/request-password-reset" "$json_data")

    if echo "$response" | grep -q "reset link"; then
        print_success "Password reset request successful"
    else
        print_error "Password reset request failed"
        echo "Response: $response"
        return 1
    fi
}

# Test logout
test_logout() {
    print_test "Testing user logout"

    if [ -z "$ACCESS_TOKEN" ]; then
        print_error "No access token available"
        return 1
    fi

    local response=$(make_request "POST" "$AUTH_BASE/user/logout" "" "Authorization: Bearer $ACCESS_TOKEN")

    if echo "$response" | grep -q "Logged out successfully"; then
        print_success "User logout successful"
    else
        print_error "User logout failed"
        echo "Response: $response"
        return 1
    fi
}

# Test invalid registration data
test_invalid_registration() {
    print_test "Testing invalid registration data"

    local json_data="{\"email\":\"invalid-email\",\"password\":\"weak\"}"
    local response=$(make_request "POST" "$AUTH_BASE/register" "$json_data")

    if echo "$response" | grep -q "validation"; then
        print_success "Invalid registration data properly rejected"
    else
        print_error "Invalid registration data not properly rejected"
        echo "Response: $response"
        return 1
    fi
}

# Test invalid login credentials
test_invalid_login() {
    print_test "Testing invalid login credentials"

    local json_data="{\"email\":\"$TEST_EMAIL\",\"password\":\"wrongpassword\"}"
    local response=$(make_request "POST" "$AUTH_BASE/login" "$json_data")

    if echo "$response" | grep -q "Invalid credentials"; then
        print_success "Invalid login credentials properly rejected"
    else
        print_error "Invalid login credentials not properly rejected"
        echo "Response: $response"
        return 1
    fi
}

# Main test runner
run_tests() {
    echo "============================================"
    echo "    Rust Web Service Authentication Tests  "
    echo "============================================"
    echo

    # Check if server is running
    if ! check_server; then
        exit 1
    fi

    local failed_tests=0
    local total_tests=0

    # Run all tests
    tests=(
        "test_health_endpoint"
        "test_invalid_registration"
        "test_user_registration"
        "test_invalid_login"
        "test_user_login"
        "test_unauthorized_access"
        "test_protected_endpoint"
        "test_password_change"
        "test_login_new_password"
        "test_token_refresh"
        "test_password_reset_request"
        "test_logout"
    )

    for test in "${tests[@]}"; do
        total_tests=$((total_tests + 1))
        echo
        if ! $test; then
            failed_tests=$((failed_tests + 1))
        fi
    done

    echo
    echo "============================================"
    echo "                Test Summary                "
    echo "============================================"
    echo "Total tests: $total_tests"
    echo "Passed: $((total_tests - failed_tests))"
    echo "Failed: $failed_tests"

    if [ $failed_tests -eq 0 ]; then
        print_success "All tests passed! 🎉"
        exit 0
    else
        print_error "$failed_tests test(s) failed"
        exit 1
    fi
}

# Cleanup function
cleanup() {
    print_info "Cleaning up test data..."
    # In a real scenario, you might want to clean up the test user
    # This would require additional admin endpoints or database access
}

# Trap cleanup on exit
trap cleanup EXIT

# Help function
show_help() {
    echo "Rust Web Service Authentication Test Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h     Show this help message"
    echo "  --url URL      Set base URL (default: http://127.0.0.1:8080)"
    echo "  --email EMAIL  Set test email (default: test@example.com)"
    echo "  --verbose, -v  Enable verbose output"
    echo ""
    echo "Examples:"
    echo "  $0                          # Run tests with default settings"
    echo "  $0 --url http://localhost:3000  # Test against different server"
    echo "  $0 --email user@test.com    # Use different test email"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_help
            exit 0
            ;;
        --url)
            BASE_URL="$2"
            API_BASE="$BASE_URL/api/v1"
            AUTH_BASE="$API_BASE/auth"
            shift 2
            ;;
        --email)
            TEST_EMAIL="$2"
            shift 2
            ;;
        --verbose|-v)
            set -x
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Run the tests
run_tests
