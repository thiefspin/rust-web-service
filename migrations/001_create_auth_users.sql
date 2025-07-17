-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create auth_users table
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

-- Create indexes for performance
CREATE INDEX idx_auth_users_email ON auth_users(email);
CREATE INDEX idx_auth_users_verification_token ON auth_users(verification_token);
CREATE INDEX idx_auth_users_reset_token ON auth_users(reset_token);
CREATE INDEX idx_auth_users_is_active ON auth_users(is_active);
CREATE INDEX idx_auth_users_created_at ON auth_users(created_at);

-- Add constraints
ALTER TABLE auth_users ADD CONSTRAINT chk_email_format
    CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$');

ALTER TABLE auth_users ADD CONSTRAINT chk_failed_attempts_non_negative
    CHECK (failed_login_attempts >= 0);

-- Create trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_auth_users_updated_at
    BEFORE UPDATE ON auth_users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE auth_users IS 'Authentication users table for storing user credentials and auth-related data';
COMMENT ON COLUMN auth_users.id IS 'Unique identifier for the user';
COMMENT ON COLUMN auth_users.email IS 'User email address (unique)';
COMMENT ON COLUMN auth_users.password_hash IS 'Bcrypt hashed password';
COMMENT ON COLUMN auth_users.is_active IS 'Whether the user account is active';
COMMENT ON COLUMN auth_users.is_verified IS 'Whether the user email has been verified';
COMMENT ON COLUMN auth_users.created_at IS 'Timestamp when the user was created';
COMMENT ON COLUMN auth_users.updated_at IS 'Timestamp when the user was last updated';
COMMENT ON COLUMN auth_users.last_login IS 'Timestamp of the last successful login';
COMMENT ON COLUMN auth_users.failed_login_attempts IS 'Number of consecutive failed login attempts';
COMMENT ON COLUMN auth_users.locked_until IS 'Timestamp until which the account is locked';
COMMENT ON COLUMN auth_users.verification_token IS 'Token for email verification';
COMMENT ON COLUMN auth_users.reset_token IS 'Token for password reset';
COMMENT ON COLUMN auth_users.reset_token_expires IS 'Expiration timestamp for password reset token';
