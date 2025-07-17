# CI/CD Documentation

This directory contains the GitHub Actions workflows and configuration files for the Rust Web Service project.

## Workflows

### 🔧 Main CI Pipeline (`ci.yml`)

The primary CI pipeline runs on every push and pull request to `main` and `develop` branches. It includes:

#### Code Quality Checks
- **Formatting**: Ensures code follows Rust formatting standards using `rustfmt`
- **Linting**: Runs Clippy with warnings treated as errors
- **Security Audit**: Checks for known security vulnerabilities using `cargo-audit`

#### Testing Matrix
- **Platforms**: Ubuntu, Windows, macOS
- **Rust Versions**: Stable, Beta, MSRV (1.70.0)
- **Database Integration**: PostgreSQL 15 for integration tests

#### Build Verification
- **Compilation**: Verifies code compiles in both debug and release modes
- **Documentation**: Ensures all documentation builds without warnings
- **Dependencies**: Checks for unused and outdated dependencies

#### Coverage Reporting
- **Code Coverage**: Generates test coverage reports using `cargo-llvm-cov`
- **Codecov Integration**: Uploads coverage data to Codecov (requires `CODECOV_TOKEN` secret)

### 🔒 Security Pipeline (`security.yml`)

Comprehensive security scanning that runs on pushes, PRs, and daily at 2 AM UTC:

#### Vulnerability Detection
- **Cargo Audit**: Scans for known security vulnerabilities in dependencies
- **Supply Chain**: Uses `cargo-deny` to check for banned/yanked crates
- **SAST**: Static Application Security Testing with CodeQL
- **Secret Scanning**: Detects hardcoded secrets using TruffleHog

#### Compliance Checks
- **License Compliance**: Verifies all dependencies use approved licenses
- **Dependency Review**: Reviews new dependencies in pull requests
- **Docker Security**: Scans Docker images with Trivy (when applicable)

### 📊 Benchmark Pipeline (`benchmark.yml`)

Performance monitoring that runs on main branch pushes and weekly:

#### Performance Testing
- **Micro-benchmarks**: Runs Criterion.rs benchmarks
- **Load Testing**: Uses k6 for HTTP load testing
- **Memory Profiling**: Monitors memory usage with Valgrind

#### Results Tracking
- **Trend Analysis**: Tracks performance over time
- **Regression Detection**: Alerts on performance degradation
- **Artifact Storage**: Preserves benchmark results for analysis

## Configuration Files

### 📦 Dependabot (`dependabot.yml`)

Automated dependency management:
- **Schedule**: Weekly updates on Mondays at 9 AM UTC
- **Grouping**: Related dependencies updated together
- **Limits**: Maximum 10 Cargo PRs, 5 GitHub Actions PRs
- **Security**: Major version updates ignored for critical dependencies

### 🚫 Cargo Deny (`deny.toml`)

Supply chain security configuration:
- **License Policy**: Allows MIT, Apache-2.0, BSD variants; denies GPL
- **Vulnerability Threshold**: Denies vulnerabilities with CVSS score > 0.1
- **Banned Crates**: Blocks known vulnerable or deprecated crates
- **Source Control**: Only allows crates.io registry

## Setup Requirements

### Required Secrets

Add these secrets in your GitHub repository settings:

```
CODECOV_TOKEN    # For code coverage reporting (optional)
```

### Required Permissions

The workflows require these GitHub token permissions:
- `security-events: write` (for security scanning)
- `actions: read` (for workflow access)
- `contents: read` (for code access)

### Environment Variables

The CI uses these environment variables for testing:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/test_db
JWT_SECRET=test_secret_key_that_is_long_enough_for_jwt_testing_in_ci
JWT_EXPIRATION=3600
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
BCRYPT_COST=4
LOG_LEVEL=debug
```

## Workflow Triggers

### Automatic Triggers
- **Push**: `main`, `develop` branches
- **Pull Request**: `main`, `develop` branches
- **Schedule**: 
  - Security scan: Daily at 2 AM UTC
  - Benchmarks: Weekly on Sunday at 3 AM UTC
  - Dependency updates: Weekly on Monday at 9 AM UTC

### Manual Triggers
All workflows can be triggered manually via the GitHub Actions UI.

## Status Badges

Add these badges to your main README.md:

```markdown
[![CI](https://github.com/your-username/rust-web-service/workflows/CI/badge.svg)](https://github.com/your-username/rust-web-service/actions/workflows/ci.yml)
[![Security](https://github.com/your-username/rust-web-service/workflows/Security/badge.svg)](https://github.com/your-username/rust-web-service/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/your-username/rust-web-service/branch/main/graph/badge.svg)](https://codecov.io/gh/your-username/rust-web-service)
```

## Troubleshooting

### Common Issues

1. **Tests failing due to missing environment variables**
   - Ensure all required environment variables are set in the workflow
   - Check that test database is properly configured

2. **Clippy warnings causing CI failure**
   - Run `cargo clippy --fix` locally to auto-fix issues
   - Add `#[allow(clippy::lint_name)]` for intentional code patterns

3. **Security audit failures**
   - Update vulnerable dependencies: `cargo update`
   - Add exceptions to `deny.toml` if necessary (with justification)

4. **Benchmark failures**
   - Ensure benchmarks are deterministic
   - Check if performance regression is expected

### Performance Optimization

- **Caching**: All workflows use `Swatinem/rust-cache` for dependency caching
- **Parallelization**: Tests run in parallel across multiple platforms
- **Selective Execution**: Some jobs only run on specific platforms or branches

## Customization

### Adding New Checks

1. Add new job to appropriate workflow file
2. Update the summary job dependencies
3. Test on a feature branch first

### Modifying Security Policies

1. Update `deny.toml` for dependency policies
2. Modify security workflow thresholds as needed
3. Consider impact on development workflow

### Performance Monitoring

1. Add custom benchmarks to the benchmark workflow
2. Configure performance regression thresholds
3. Set up notifications for significant changes

## Best Practices

1. **Keep workflows fast**: Use caching and parallel execution
2. **Fail fast**: Put quick checks (fmt, clippy) first
3. **Secure by default**: All security checks should fail on issues
4. **Monitor performance**: Track trends over time
5. **Document changes**: Update this README when modifying workflows