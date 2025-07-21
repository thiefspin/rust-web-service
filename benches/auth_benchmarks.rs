use bcrypt::{hash, verify};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_web_service::models::auth_user::{LoginRequest, RegisterRequest};
use rust_web_service::services::auth_service::AuthService;
use uuid::Uuid;

fn bcrypt_benchmark(c: &mut Criterion) {
    let password = "test_password_123";

    c.bench_function("bcrypt_hash_cost_4", |b| {
        b.iter(|| hash(black_box(password), black_box(4)).unwrap())
    });

    c.bench_function("bcrypt_hash_cost_12", |b| {
        b.iter(|| hash(black_box(password), black_box(12)).unwrap())
    });

    // Pre-compute hash for verification benchmark
    let hashed = hash(password, 4).unwrap();

    c.bench_function("bcrypt_verify", |b| {
        b.iter(|| verify(black_box(password), black_box(&hashed)).unwrap())
    });
}

fn jwt_benchmark(c: &mut Criterion) {
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        email: String,
        exp: usize,
    }

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        email: "test@example.com".to_string(),
        exp: 10000000000, // Future timestamp
    };

    let secret = "test_secret_key_that_is_long_enough_for_jwt_testing";
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let decoding_key = DecodingKey::from_secret(secret.as_ref());

    c.bench_function("jwt_encode", |b| {
        b.iter(|| {
            encode(
                &Header::default(),
                black_box(&claims),
                black_box(&encoding_key),
            )
            .unwrap()
        })
    });

    let token = encode(&Header::default(), &claims, &encoding_key).unwrap();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false; // Disable expiration validation for benchmark

    c.bench_function("jwt_decode", |b| {
        b.iter(|| {
            decode::<Claims>(
                black_box(&token),
                black_box(&decoding_key),
                black_box(&validation),
            )
            .unwrap()
        })
    });
}

fn uuid_benchmark(c: &mut Criterion) {
    c.bench_function("uuid_v4_generation", |b| b.iter(|| Uuid::new_v4()));

    let uuid = Uuid::new_v4();
    let uuid_string = uuid.to_string();

    c.bench_function("uuid_to_string", |b| b.iter(|| black_box(uuid).to_string()));

    c.bench_function("uuid_from_string", |b| {
        b.iter(|| Uuid::parse_str(black_box(&uuid_string)).unwrap())
    });
}

fn validation_benchmark(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};
    use validator::{Validate, ValidationErrors};

    #[derive(Debug, Serialize, Deserialize, Validate)]
    struct TestEmail {
        #[validate(email)]
        email: String,
    }

    let valid_email = TestEmail {
        email: "test@example.com".to_string(),
    };

    let invalid_email = TestEmail {
        email: "invalid-email".to_string(),
    };

    c.bench_function("email_validation_valid", |b| {
        b.iter(|| black_box(&valid_email).validate())
    });

    c.bench_function("email_validation_invalid", |b| {
        b.iter(|| {
            let _ = black_box(&invalid_email).validate();
        })
    });
}

fn serialization_benchmark(c: &mut Criterion) {
    use serde_json;

    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "Test123!@#".to_string(),
    };

    c.bench_function("serde_json_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&register_request)).unwrap())
    });

    let json_string = serde_json::to_string(&register_request).unwrap();

    c.bench_function("serde_json_deserialize", |b| {
        b.iter(|| serde_json::from_str::<RegisterRequest>(black_box(&json_string)).unwrap())
    });

    c.bench_function("serde_json_serialize_pretty", |b| {
        b.iter(|| serde_json::to_string_pretty(black_box(&register_request)).unwrap())
    });
}

fn string_operations_benchmark(c: &mut Criterion) {
    let email = "test@example.com";
    let domain = "example.com";

    c.bench_function("string_clone", |b| b.iter(|| black_box(email).to_string()));

    c.bench_function("string_contains", |b| {
        b.iter(|| black_box(email).contains(black_box(domain)))
    });

    c.bench_function("string_split", |b| {
        b.iter(|| black_box(email).split('@').collect::<Vec<&str>>())
    });

    let password = "Test123!@#";
    c.bench_function("string_length_check", |b| {
        b.iter(|| black_box(password).len() >= 8)
    });
}

fn regex_benchmark(c: &mut Criterion) {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static EMAIL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

    let valid_email = "test@example.com";
    let invalid_email = "invalid.email";

    c.bench_function("regex_email_match_valid", |b| {
        b.iter(|| EMAIL_REGEX.is_match(black_box(valid_email)))
    });

    c.bench_function("regex_email_match_invalid", |b| {
        b.iter(|| EMAIL_REGEX.is_match(black_box(invalid_email)))
    });

    // Test regex compilation cost
    c.bench_function("regex_compile", |b| {
        b.iter(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap())
    });
}

criterion_group!(
    benches,
    bcrypt_benchmark,
    jwt_benchmark,
    uuid_benchmark,
    validation_benchmark,
    serialization_benchmark,
    string_operations_benchmark,
    regex_benchmark
);

criterion_main!(benches);
