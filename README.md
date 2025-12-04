# Rust Tutorial API

Minimal Actix-web API that demonstrates user registration and login with SeaORM (Postgres) and Argon2 password hashing. A companion migration crate manages the `auth_users` table, and Docker Compose is available for a local app + database stack.

## Stack
- Rust 2024 with Actix-web
- SeaORM + Postgres, chrono
- Argon2 password hashing
- Dockerfile + docker-compose for containerized runs

## Prerequisites
- Rust toolchain (cargo)
- Postgres database reachable via `DATABASE_URL`
- Docker and Docker Compose (optional, for containerized setup)

## Setup (local)
1) Copy `.env.example` to `.env` and update `DATABASE_URL`, `HOST`, and `PORT` to match your environment.
2) Ensure Postgres is running and the database in `DATABASE_URL` exists.
3) Run migrations:
   ```sh
   cd migration
   cargo run -- up
   ```
4) Start the API:
   ```sh
   cargo run
   ```
   The server listens on `HOST:PORT` (defaults to `127.0.0.1:8080`).

## Running with Docker Compose
```sh
docker-compose up --build
```
- App: http://localhost:8080
- DB: exposed on port 5432 with credentials from `.env` or defaults in `docker-compose.yml`.

## API Endpoints
- `GET /` — health check, returns "Hello world!".
- `POST /api/v1/auth/register` — create a user. Example:
  ```sh
  curl -X POST http://localhost:8080/api/v1/auth/register \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","password":"secret","email":"alice@example.com","phone":"0800000000"}'
  ```
- `POST /api/v1/auth/login` — verify credentials. Example:
  ```sh
  curl -X POST http://localhost:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","password":"secret"}'
  ```
- `GET /api/v1/auth/profile` — protected route, requires `Authorization: Bearer <JWT>` from the login response. Example:
  ```sh
  curl http://localhost:8080/api/v1/auth/profile \
    -H "Authorization: Bearer <token-from-login>"
  ```

## Data Model
`auth_users` columns: `id`, `username`, `password` (Argon2 hash), `email`, `phone`, `active`, `created_at`, `updated_at`.

## Project Layout
- `src/` — server, routes, handlers, models
- `migration/` — SeaORM migration crate (`cargo run -- up` to apply)
- `Dockerfile`, `docker-compose.yml` — container builds and services

## Notes
- Passwords are hashed with Argon2 before storage.
- `JWT_SECRET` must be set for JWT signing/verification. You can generate a 32-byte base64 key in PowerShell:
  ```powershell
  $bytes = New-Object byte[] 32; [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes); [Convert]::ToBase64String($bytes)
  ```
