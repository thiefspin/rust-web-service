use actix_web::test::{call_service, init_service, TestRequest};
use actix_web::{web, App, HttpResponse, Result as ActixResult};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

// Mock handlers for benchmarking
async fn health_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn echo_handler(body: web::Json<serde_json::Value>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(&*body))
}

async fn large_response_handler() -> ActixResult<HttpResponse> {
    let large_data = (0..1000)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("Item {}", i),
                "description": "This is a test item with some description text",
                "tags": ["tag1", "tag2", "tag3"],
                "active": i % 2 == 0,
                "created_at": chrono::Utc::now().to_rfc3339()
            })
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(json!({
        "data": large_data,
        "total": 1000,
        "page": 1,
        "per_page": 1000
    })))
}

fn http_handlers_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("health_endpoint", |b| {
        b.to_async(&rt).iter(|| async {
            let app =
                init_service(App::new().route("/health", web::get().to(health_handler))).await;

            let req = TestRequest::get().uri("/health").to_request();
            let resp = call_service(&app, req).await;
            black_box(resp);
        });
    });

    let test_data = json!({
        "message": "Hello, World!",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "id": 123,
            "values": [1, 2, 3, 4, 5]
        }
    });

    c.bench_function("echo_endpoint", |b| {
        b.to_async(&rt).iter(|| {
            let data = test_data.clone();
            async move {
                let app =
                    init_service(App::new().route("/echo", web::post().to(echo_handler))).await;

                let req = TestRequest::post()
                    .uri("/echo")
                    .set_json(&data)
                    .to_request();
                let resp = call_service(&app, req).await;
                black_box(resp);
            }
        });
    });

    c.bench_function("large_response_endpoint", |b| {
        b.to_async(&rt).iter(|| async {
            let app =
                init_service(App::new().route("/large", web::get().to(large_response_handler)))
                    .await;

            let req = TestRequest::get().uri("/large").to_request();
            let resp = call_service(&app, req).await;
            black_box(resp);
        });
    });
}

fn json_processing_benchmark(c: &mut Criterion) {
    let small_json = json!({
        "id": 1,
        "name": "Test",
        "active": true
    });

    let medium_json = json!({
        "id": 1,
        "name": "Test User",
        "email": "test@example.com",
        "profile": {
            "age": 30,
            "city": "New York",
            "preferences": {
                "theme": "dark",
                "notifications": true,
                "language": "en"
            }
        },
        "tags": ["user", "premium", "active"],
        "metadata": {
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-12-01T00:00:00Z",
            "version": 2
        }
    });

    let large_json = json!({
        "users": (0..100).map(|i| json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "posts": (0..10).map(|j| json!({
                "id": j,
                "title": format!("Post {} by User {}", j, i),
                "content": "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                "tags": ["tag1", "tag2", "tag3"]
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    });

    c.bench_function("json_serialize_small", |b| {
        b.iter(|| serde_json::to_string(black_box(&small_json)).unwrap())
    });

    c.bench_function("json_serialize_medium", |b| {
        b.iter(|| serde_json::to_string(black_box(&medium_json)).unwrap())
    });

    c.bench_function("json_serialize_large", |b| {
        b.iter(|| serde_json::to_string(black_box(&large_json)).unwrap())
    });

    let small_string = serde_json::to_string(&small_json).unwrap();
    let medium_string = serde_json::to_string(&medium_json).unwrap();
    let large_string = serde_json::to_string(&large_json).unwrap();

    c.bench_function("json_deserialize_small", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&small_string)).unwrap())
    });

    c.bench_function("json_deserialize_medium", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&medium_string)).unwrap())
    });

    c.bench_function("json_deserialize_large", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&large_string)).unwrap())
    });
}

fn http_request_parsing_benchmark(c: &mut Criterion) {
    use actix_web::test::TestRequest;

    c.bench_function("parse_get_request", |b| {
        b.iter(|| {
            let req = TestRequest::get()
                .uri("/api/v1/users?page=1&limit=10&sort=name")
                .insert_header(("User-Agent", "test-client/1.0"))
                .insert_header(("Accept", "application/json"))
                .to_request();
            black_box(req);
        })
    });

    c.bench_function("parse_post_request_with_json", |b| {
        b.iter(|| {
            let req = TestRequest::post()
                .uri("/api/v1/users")
                .insert_header(("Content-Type", "application/json"))
                .insert_header(("User-Agent", "test-client/1.0"))
                .set_json(&json!({
                    "name": "John Doe",
                    "email": "john@example.com",
                    "age": 30
                }))
                .to_request();
            black_box(req);
        })
    });

    c.bench_function("parse_request_with_many_headers", |b| {
        b.iter(|| {
            let req = TestRequest::get()
                .uri("/api/v1/data")
                .insert_header((
                    "Authorization",
                    "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9",
                ))
                .insert_header(("User-Agent", "Mozilla/5.0 (compatible; test-client/1.0)"))
                .insert_header(("Accept", "application/json,text/html;q=0.9"))
                .insert_header(("Accept-Language", "en-US,en;q=0.8"))
                .insert_header(("Accept-Encoding", "gzip, deflate"))
                .insert_header(("Connection", "keep-alive"))
                .insert_header(("Cache-Control", "no-cache"))
                .insert_header(("X-Requested-With", "XMLHttpRequest"))
                .insert_header(("X-Client-Version", "1.2.3"))
                .insert_header(("X-Request-ID", "req-123456789"))
                .to_request();
            black_box(req);
        })
    });
}

fn response_building_benchmark(c: &mut Criterion) {
    c.bench_function("build_simple_response", |b| {
        b.iter(|| {
            let response = HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Operation completed"
            }));
            black_box(response);
        })
    });

    c.bench_function("build_response_with_headers", |b| {
        b.iter(|| {
            let response = HttpResponse::Ok()
                .insert_header(("X-Response-Time", "10ms"))
                .insert_header(("X-Rate-Limit", "1000"))
                .insert_header(("X-Rate-Remaining", "999"))
                .insert_header(("Cache-Control", "public, max-age=300"))
                .json(json!({
                    "data": {
                        "id": 123,
                        "name": "Test Item"
                    }
                }));
            black_box(response);
        })
    });

    let large_data = (0..50)
        .map(|i| {
            json!({
                "id": i,
                "title": format!("Item {}", i),
                "description": "A sample description with some text content",
                "metadata": {
                    "created": "2023-01-01T00:00:00Z",
                    "tags": ["tag1", "tag2"]
                }
            })
        })
        .collect::<Vec<_>>();

    c.bench_function("build_large_response", |b| {
        b.iter(|| {
            let response = HttpResponse::Ok().json(json!({
                "data": &large_data,
                "pagination": {
                    "page": 1,
                    "per_page": 50,
                    "total": 50,
                    "has_more": false
                }
            }));
            black_box(response);
        })
    });
}

fn url_parsing_benchmark(c: &mut Criterion) {
    let simple_url = "/api/v1/users";
    let url_with_query = "/api/v1/users?page=1&limit=10&sort=name&filter=active";
    let complex_url = "/api/v1/organizations/123/departments/456/users?include=profile,permissions&sort=name,email&filter[status]=active&filter[role]=admin&page=2&per_page=25";

    c.bench_function("parse_simple_url", |b| {
        b.iter(|| {
            let parts: Vec<&str> = black_box(simple_url).split('/').collect();
            black_box(parts);
        })
    });

    c.bench_function("parse_url_with_query", |b| {
        b.iter(|| {
            let url = black_box(url_with_query);
            if let Some((path, query)) = url.split_once('?') {
                let path_parts: Vec<&str> = path.split('/').collect();
                let query_parts: Vec<&str> = query.split('&').collect();
                black_box((path_parts, query_parts));
            }
        })
    });

    c.bench_function("parse_complex_url", |b| {
        b.iter(|| {
            let url = black_box(complex_url);
            if let Some((path, query)) = url.split_once('?') {
                let path_parts: Vec<&str> = path.split('/').collect();
                let query_params: std::collections::HashMap<&str, &str> = query
                    .split('&')
                    .filter_map(|param| param.split_once('='))
                    .collect();
                black_box((path_parts, query_params));
            }
        })
    });
}

criterion_group!(
    benches,
    http_handlers_benchmark,
    json_processing_benchmark,
    http_request_parsing_benchmark,
    response_building_benchmark,
    url_parsing_benchmark
);

criterion_main!(benches);
