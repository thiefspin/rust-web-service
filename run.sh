#!/bin/bash

# Development run script for Rust Web Service
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    print_status "Checking dependencies..."

    if ! command_exists cargo; then
        print_error "Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi

    if ! command_exists psql; then
        print_warning "PostgreSQL client not found. Make sure PostgreSQL is installed."
    fi

    if ! command_exists sqlx; then
        print_warning "sqlx-cli not found. Installing..."
        cargo install sqlx-cli --no-default-features --features postgres
    fi

    print_success "Dependencies check completed"
}

# Function to setup environment
setup_env() {
    print_status "Setting up environment..."

    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            cp .env.example .env
            print_warning "Created .env from .env.example. Please edit .env with your configuration."
            echo "Edit the .env file and run this script again."
            exit 1
        else
            print_error ".env.example not found. Please create .env file manually."
            exit 1
        fi
    fi

    # Load environment variables
    export $(cat .env | grep -v '^#' | xargs)

    print_success "Environment setup completed"
}

# Function to setup database
setup_database() {
    print_status "Setting up database..."

    if [ -z "$DATABASE_URL" ]; then
        print_error "DATABASE_URL not set in .env file"
        exit 1
    fi

    # Test database connection
    print_status "Testing database connection..."
    if ! sqlx database create 2>/dev/null; then
        print_warning "Database might already exist or connection failed"
    fi

    # Run migrations
    print_status "Running database migrations..."
    if sqlx migrate run; then
        print_success "Database migrations completed"
    else
        print_error "Database migrations failed"
        exit 1
    fi
}

# Function to run tests
run_tests() {
    print_status "Running tests..."

    if cargo test; then
        print_success "All tests passed"
    else
        print_error "Some tests failed"
        exit 1
    fi
}

# Function to start the application
start_application() {
    print_status "Starting the application..."

    # Check if we should run in development mode
    if [ "$1" = "--dev" ] || [ "$1" = "-d" ]; then
        print_status "Running in development mode with file watching..."
        if command_exists cargo-watch; then
            cargo watch -x run
        else
            print_warning "cargo-watch not found. Installing..."
            cargo install cargo-watch
            cargo watch -x run
        fi
    else
        cargo run
    fi
}

# Function to build the application
build_application() {
    print_status "Building the application..."

    if [ "$1" = "--release" ] || [ "$1" = "-r" ]; then
        cargo build --release
        print_success "Release build completed"
    else
        cargo build
        print_success "Debug build completed"
    fi
}

# Function to show help
show_help() {
    echo "Rust Web Service - Development Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  setup       Setup environment and database"
    echo "  run         Run the application (default)"
    echo "  test        Run tests"
    echo "  build       Build the application"
    echo "  clean       Clean build artifacts"
    echo "  fmt         Format code"
    echo "  lint        Run clippy linter"
    echo "  help        Show this help message"
    echo ""
    echo "Options:"
    echo "  --dev, -d   Run with file watching (for run command)"
    echo "  --release, -r  Build in release mode (for build command)"
    echo ""
    echo "Examples:"
    echo "  $0 setup           # Setup environment and database"
    echo "  $0 run --dev       # Run with file watching"
    echo "  $0 build --release # Build in release mode"
    echo "  $0 test            # Run tests"
}

# Main script logic
case "$1" in
    "setup")
        check_dependencies
        setup_env
        setup_database
        print_success "Setup completed successfully!"
        ;;
    "run"|"")
        check_dependencies
        setup_env
        setup_database
        start_application "$2"
        ;;
    "test")
        check_dependencies
        setup_env
        run_tests
        ;;
    "build")
        check_dependencies
        build_application "$2"
        ;;
    "clean")
        print_status "Cleaning build artifacts..."
        cargo clean
        print_success "Clean completed"
        ;;
    "fmt")
        print_status "Formatting code..."
        cargo fmt
        print_success "Code formatting completed"
        ;;
    "lint")
        print_status "Running clippy linter..."
        cargo clippy -- -D warnings
        print_success "Linting completed"
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
