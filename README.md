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
- Docker & Docker Compose (optional, for containerized setup)

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
- `GET /` – health check, returns "Hello world!".
- `POST /register` – create a user. Example:
  ```sh
  curl -X POST http://localhost:8080/register \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","password":"secret","email":"alice@example.com","phone":"0800000000"}'
  ```
- `POST /login` – verify credentials. Example:
  ```sh
  curl -X POST http://localhost:8080/login \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","password":"secret"}'
  ```

## Data Model
`auth_users` columns: `id`, `username`, `password` (Argon2 hash), `email`, `phone`, `active`, `created_at`, `updated_at`.

## Project Layout
- `src/` – server, routes, handlers, models
- `migration/` – SeaORM migration crate (`cargo run -- up` to apply)
- `Dockerfile`, `docker-compose.yml` – container builds and services

## Notes
- Passwords are hashed with Argon2 before storage.
- The sample `.env.example` includes extra keys for convenience; only `DATABASE_URL`, `HOST`, and `PORT` are required by the current code.
