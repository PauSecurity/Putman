use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use chrono::{Duration, Utc};

use jsonwebtoken::{
    decode,
    encode,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
};

use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    env,
    sync::Arc,
};

use tower_http::trace::TraceLayer;

const DEFAULT_JWT_SECRET: &str = "super-secret-test-key";

#[derive(Clone)]
struct AppState {
    jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Params {
    name: Option<String>,
    age: Option<u32>,
}

#[derive(Debug, Serialize)]
struct EchoResponse {
    method: String,
    headers: HashMap<String, String>,
    query: HashMap<String, String>,
    body: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let jwt_secret =
        env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string());

    let state = Arc::new(AppState { jwt_secret });

    let app = Router::new()
        // Basic endpoints
        .route("/", get(root))
        .route("/health", get(health))

        // Parameters
        .route("/params", get(params))

        // Headers
        .route("/headers", get(headers))

        // Path parameters
        .route("/users/{id}", get(user_by_id))

        // Echo request
        .route("/echo", post(echo))

        // Authentication
        .route("/login", post(login))

        // Protected endpoint
        .route("/protected", get(protected))

        // Logging middleware
        .layer(TraceLayer::new_for_http())

        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Could not bind to port 8080");

    println!("======================================");
    println!(" API Test Server");
    println!("======================================");
    println!("Server running on:");
    println!("http://localhost:8080");
    println!("======================================");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// ======================================
// GET /
// ======================================

async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "API Test Server is running",
        "version": "1.0.0"
    }))
}

// ======================================
// GET /health
// ======================================

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

// ======================================
// GET /params
//
// Example:
// /params?name=Joaquin&age=23
// ======================================

async fn params(Query(params): Query<Params>) -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "Parameters received",
        "name": params.name,
        "age": params.age
    }))
}

// ======================================
// GET /headers
// ======================================

async fn headers(headers: HeaderMap) -> impl IntoResponse {
    let mut result = HashMap::new();

    for (name, value) in headers.iter() {
        if let Ok(value) = value.to_str() {
            result.insert(name.to_string(), value.to_string());
        }
    }

    Json(result)
}

// ======================================
// GET /users/:id
//
// Example:
// /users/123
// ======================================

async fn user_by_id(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "User found",
        "user_id": id
    }))
}

// ======================================
// POST /echo
//
// Returns information about the request
// ======================================

async fn echo(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut header_map = HashMap::new();

    for (name, value) in headers.iter() {
        if let Ok(value) = value.to_str() {
            header_map.insert(name.to_string(), value.to_string());
        }
    }

    Json(EchoResponse {
        method: "POST".to_string(),
        headers: header_map,
        query,
        body: Some(body),
    })
}

// ======================================
// POST /login
//
// Example body:
//
// {
//   "username": "joaquin",
//   "password": "123456"
// }
// ======================================

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Fake authentication.
    // This is only a test server.
    if payload.username != "joaquin" || payload.password != "123456" {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid username or password"
            })),
        );
    }

    let expiration = Utc::now() + Duration::hours(1);

    let claims = Claims {
        sub: "123".to_string(),
        username: payload.username,
        exp: expiration.timestamp() as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ) {
        Ok(token) => token,

        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Could not generate token"
                })),
            );
        }
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "access_token": token,
            "token_type": "Bearer",
            "expires_in": 3600
        })),
    )
}

// ======================================
// GET /protected
//
// Requires:
//
// Authorization: Bearer <JWT>
// ======================================

async fn protected(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let authorization = match headers.get("Authorization") {
        Some(value) => match value.to_str() {
            Ok(value) => value,

            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Invalid Authorization header"
                    })),
                );
            }
        },

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing Authorization header"
                })),
            );
        }
    };

    // Expected:
    //
    // Authorization: Bearer eyJ...

    let token = match authorization.strip_prefix("Bearer ") {
        Some(token) => token,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Authorization must use Bearer token"
                })),
            );
        }
    };

    let validation = Validation::default();

    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    );

    match decoded {
        Ok(token_data) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Authentication successful",
                "user": {
                    "id": token_data.claims.sub,
                    "username": token_data.claims.username
                }
            })),
        ),

        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid or expired JWT"
            })),
        ),
    }
}