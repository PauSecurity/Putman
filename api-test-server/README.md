# API Test Server

A small HTTP API server built with **Rust + Axum**, designed to test the Postman-like application.

The server provides endpoints for testing:

- HTTP requests
- Query parameters
- Path parameters
- Request headers
- JSON request bodies
- Bearer authentication
- JWT authentication
- Protected routes
- HTTP status codes
- Docker-based execution

---

## Requirements

You only need:

- Docker
- Docker Compose

You **do not need Rust or Cargo installed** to run the server with Docker.

---

## Quick Start

### 1. Enter the test server directory

```bash
cd test-server
```

### 2. Start the server

```bash
docker compose up --build
```

The server will start on:

```text
http://localhost:8080
```

You should see:

```text
======================================
 API Test Server
======================================
Server running on:
http://localhost:8080
======================================
```

Keep this terminal running while testing the API.

---

## Stop the Server

Press:

```text
Ctrl + C
```

Or, from another terminal:

```bash
docker compose down
```

---

# API Endpoints

Base URL:

```text
http://localhost:8080
```

---

## 1. Health Check

### Request

```http
GET /health
```

### cURL

```bash
curl http://localhost:8080/health
```

### Response

```json
{
  "status": "ok"
}
```

---

# 2. Root Endpoint

### Request

```http
GET /
```

### cURL

```bash
curl http://localhost:8080/
```

### Response

```json
{
  "message": "API Test Server is running",
  "version": "1.0.0"
}
```

---

# 3. Query Parameters

This endpoint is useful for testing query parameters.

### Request

```http
GET /params?name=Joaquin&age=23
```

### cURL

```bash
curl "http://localhost:8080/params?name=Joaquin&age=23"
```

### Response

```json
{
  "message": "Parameters received",
  "name": "Joaquin",
  "age": 23
}
```

### Parameters

| Parameter | Type   | Example   |
| --------- | ------ | --------- |
| `name`    | string | `Joaquin` |
| `age`     | number | `23`      |

You can also test different values:

```text
http://localhost:8080/params?name=Aziz&age=25
```

---

# 4. Request Headers

This endpoint returns the headers received by the server.

### Request

```http
GET /headers
```

### cURL

```bash
curl \
  -H "X-Test: Hello" \
  -H "X-User: Joaquin" \
  http://localhost:8080/headers
```

### Response

The response will contain the received headers:

```json
{
  "x-test": "Hello",
  "x-user": "Joaquin"
}
```

This endpoint is useful for testing the application's **Headers** section.

---

# 5. Path Parameters

This endpoint tests URL/path parameters.

### Request

```http
GET /users/{id}
```

Example:

```http
GET /users/123
```

### cURL

```bash
curl http://localhost:8080/users/123
```

### Response

```json
{
  "message": "User found",
  "user_id": "123"
}
```

Try different IDs:

```text
/users/1
/users/42
/users/999
/users/aziz
```

---

# 6. POST Request + JSON Body + Headers + Query Parameters

The `/echo` endpoint is designed to test several features at the same time.

It accepts:

- Query parameters
- Request headers
- JSON body

### Request

```http
POST /echo
```

Example URL:

```text
http://localhost:8080/echo?test=true&name=Joaquin
```

### cURL

```bash
curl -X POST \
  "http://localhost:8080/echo?test=true&name=Joaquin" \
  -H "Content-Type: application/json" \
  -H "X-Test: Hello" \
  -H "X-Client: MyPostman" \
  -d '{
    "message": "Hello from the client",
    "number": 123,
    "active": true
  }'
```

### Example Response

```json
{
  "method": "POST",
  "headers": {
    "content-type": "application/json",
    "x-test": "Hello",
    "x-client": "MyPostman"
  },
  "query": {
    "test": "true",
    "name": "Joaquin"
  },
  "body": {
    "message": "Hello from the client",
    "number": 123,
    "active": true
  }
}
```

This is one of the most useful endpoints for testing the application.

---

# 7. Login / Generate JWT

The `/login` endpoint generates a JWT token.

### Request

```http
POST /login
```

### URL

```text
http://localhost:8080/login
```

### Headers

```http
Content-Type: application/json
```

### Request Body

```json
{
  "username": "joaquin",
  "password": "123456"
}
```

### cURL

```bash
curl -X POST \
  http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "joaquin",
    "password": "123456"
  }'
```

### Successful Response

```json
{
  "access_token": "YOUR_JWT_TOKEN",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

The actual `access_token` will be different every time.

---

## Login Credentials

For testing purposes:

```text
Username: joaquin
Password: 123456
```

These are **test credentials only**.

Do not use them for a real application.

---

# 8. Invalid Login

You can also test an authentication failure.

### cURL

```bash
curl -X POST \
  http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "wrong",
    "password": "wrong"
  }'
```

### Response

HTTP status:

```text
401 Unauthorized
```

Response body:

```json
{
  "error": "Invalid username or password"
}
```

---

# 9. Protected Endpoint

The `/protected` endpoint requires a valid JWT.

### Request

```http
GET /protected
```

You must send:

```http
Authorization: Bearer YOUR_JWT_TOKEN
```

---

## Step 1 — Get a Token

Run:

```bash
curl -X POST \
  http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "joaquin",
    "password": "123456"
  }'
```

Copy the value of:

```json
"access_token"
```

---

## Step 2 — Send the Token

Replace `YOUR_JWT_TOKEN` with the token you received:

```bash
curl \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  http://localhost:8080/protected
```

### Successful Response

```json
{
  "message": "Authentication successful",
  "user": {
    "id": "123",
    "username": "joaquin"
  }
}
```

---

# 10. Protected Endpoint Without Token

You can test what happens when authentication is missing.

### cURL

```bash
curl http://localhost:8080/protected
```

### Response

HTTP status:

```text
401 Unauthorized
```

```json
{
  "error": "Missing Authorization header"
}
```

---

# 11. Protected Endpoint With Invalid Token

You can also test an invalid Bearer token:

```bash
curl \
  -H "Authorization: Bearer invalid-token" \
  http://localhost:8080/protected
```

### Response

HTTP status:

```text
401 Unauthorized
```

```json
{
  "error": "Invalid or expired JWT"
}
```

---

# JWT Configuration

The JWT secret is provided through the `JWT_SECRET` environment variable.

Docker Compose currently uses:

```yaml
environment:
  JWT_SECRET: "super-secret-test-key"
```

This is intentionally a simple test secret.

**Do not use this secret in production.**

If necessary, the secret can be changed in `docker-compose.yml`.

---

# Testing With the Postman-Like Application

The main purpose of this server is to test our application.

Use:

```text
http://localhost:8080
```

as the base URL.

Some useful requests to create in the application:

### Basic GET

```text
GET http://localhost:8080/health
```

### Query Parameters

```text
GET http://localhost:8080/params
```

Parameters:

```text
name = Joaquin
age = 23
```

### Headers

```text
GET http://localhost:8080/headers
```

Headers:

```text
X-Test: Hello
X-User: Joaquin
```

### Path Parameter

```text
GET http://localhost:8080/users/123
```

### JSON Body

```text
POST http://localhost:8080/echo?test=true
```

Headers:

```text
Content-Type: application/json
```

Body:

```json
{
  "message": "Hello",
  "number": 123,
  "active": true
}
```

### JWT Login

```text
POST http://localhost:8080/login
```

Body:

```json
{
  "username": "joaquin",
  "password": "123456"
}
```

### JWT Protected Request

```text
GET http://localhost:8080/protected
```

Header:

```text
Authorization: Bearer YOUR_JWT_TOKEN
```

---

# Useful Test Scenarios

The following scenarios should be tested in the Postman-like application:

| Feature                | Endpoint          |
| ---------------------- | ----------------- |
| Basic GET              | `GET /`           |
| Health check           | `GET /health`     |
| Query parameters       | `GET /params`     |
| Request headers        | `GET /headers`    |
| Path parameters        | `GET /users/{id}` |
| JSON body              | `POST /echo`      |
| Query + headers + body | `POST /echo`      |
| JWT login              | `POST /login`     |
| Invalid credentials    | `POST /login`     |
| Bearer authentication  | `GET /protected`  |
| Missing authentication | `GET /protected`  |
| Invalid JWT            | `GET /protected`  |

---

# Troubleshooting

## Port 8080 is already in use

If Docker reports that port `8080` is already being used, check what is using it:

```bash
lsof -i :8080
```

You can stop the process or change the port in `docker-compose.yml`.

For example:

```yaml
ports:
  - "8081:8080"
```

Then access the server at:

```text
http://localhost:8081
```

---

## Docker container is not running

Check the containers:

```bash
docker ps
```

Check all containers:

```bash
docker ps -a
```

View the server logs:

```bash
docker compose logs
```

---

## Rebuild the server

If the Rust code or Docker configuration has changed:

```bash
docker compose down
docker compose up --build
```

---

# Development Without Docker

Docker is recommended for normal testing.

If you have Rust installed, you can also run the server directly:

```bash
cargo run
```

The server will still listen on:

```text
http://localhost:8080
```

However, Docker is the recommended way to ensure that both developers are running the same environment.

---

## Project Structure

```text
test-server/
├── src/
│   └── main.rs
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── .dockerignore
└── README.md
```

---

## Summary

For normal use, simply run:

```bash
cd test-server
docker compose up --build
```

Then use:

```text
http://localhost:8080
```

to test the Postman-like application.

The server is intentionally simple and exists only as a controlled API environment for development and testing.
