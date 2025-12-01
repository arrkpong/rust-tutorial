// src/handler/auth_handler.rs
use crate::models::auth_model::{ActiveModel, Column, Entity, LoginRequest, RegisterRequest};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use argon2::password_hash::SaltString;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, rand_core::OsRng},
};
use sea_orm::Condition;
use sea_orm::DatabaseConnection;
use sea_orm::entity::prelude::*;
use sea_orm::error::SqlErr;
use serde_json::json;
use tracing::{debug, error, info, warn};
use validator::Validate;

//===============================
// Actix-web Handlers
//===============================
#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({"code":200,"message":"Hello world!"}))
}

#[post("/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    form: web::Json<LoginRequest>,
) -> impl Responder {
    if let Err(e) = form.validate() {
        warn!("Validation error during login: {:?}", e);
        return HttpResponse::BadRequest()
            .json(json!({"code":400,"message":"Validation error","errors":e}));
    }

    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // 1. Database Query (Async I/O Bound)
    // This runs on the main async thread pool. It yields control while waiting for the DB.
    // Main Thread / Async
    let user = match Entity::find()
        .filter(Column::Username.eq(&form.username))
        .filter(Column::Active.eq(true))
        .one(db.get_ref())
        .await
    {
        Ok(Some(res)) => {
            debug!("User found: {}", res.username);
            res
        }
        Ok(None) => {
            warn!(
                "Login failed: user {} not found from IP {}",
                form.username, client_ip
            );
            return HttpResponse::Unauthorized()
                .json(json!({"code":401,"message":"invalid credentials"}));
        }
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"code":500,"message":"Internal server error"}));
        }
    };

    // 2. Prepare Data for the Blocking Thread
    // We must clone the data because we are sending it to a separate thread.
    // Rust requires 'Owned' data to be moved into the closure, as references cannot safe-cross thread boundaries here.
    let password_input = form.password.clone();
    let password_hash_stored = user.password.clone();

    // 3. CPU Intensive Task (Argon2 Verification)
    // We offload this to `web::block`, which runs on a separate thread pool dedicated to blocking operations.
    // This prevents the main async worker threads from freezing during the heavy calculation.
    // Blocking Thread / Sync
    let verify_result = web::block(move || {
        // --- Inside Blocking Thread ---

        // 3.1 Parse the stored hash string into a PasswordHash object
        let parsed_hash = match PasswordHash::new(&password_hash_stored) {
            Ok(hash) => hash,
            Err(e) => return Err(format!("Password hash parsing error: {}", e)),
        };

        // 3.2 Verify the input password against the stored hash
        match Argon2::default().verify_password(password_input.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(()),
            Err(_) => Err("Invalid password".to_string()),
        }
    })
    .await;

    // 4. Handle the Nested Result (Unwrapping the layers)
    match verify_result {
        // Outer Layer (Ok): The task was successfully executed by the thread pool.
        Ok(inner_result) => match inner_result {
            // Inner Layer (Ok): Password verification succeeded.
            Ok(()) => {
                info!(
                    "User {} logged in successfully from IP {}",
                    form.username, client_ip
                );
                HttpResponse::Ok().json(json!({"code":200,"message":"Login successful"}))
            }

            // Inner Layer (Err): Logic error (Wrong password or Malformed hash).
            Err(err_msg) => {
                if err_msg.contains("parsing error") {
                    error!("{}", err_msg);
                    HttpResponse::InternalServerError()
                        .json(json!({"code":500,"message":"Internal server error"}))
                } else {
                    warn!(
                        "Login failed: invalid password for user {} from IP {}",
                        form.username, client_ip
                    );
                    HttpResponse::Unauthorized()
                        .json(json!({"code":401,"message":"invalid credentials"}))
                }
            }
        },

        // Outer Layer (Err): The thread pool failed to execute the task (e.g., Pool overloaded or Cancelled).
        Err(e) => {
            error!("Blocking execution error (Thread pool issue): {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"code":500,"message":"Internal server error"}))
        }
    }
}

#[post("/register")]
pub async fn register(
    db: web::Data<DatabaseConnection>,
    form: web::Json<RegisterRequest>,
) -> impl Responder {
    if let Err(e) = form.validate() {
        warn!("Validation error during registration: {:?}", e);
        return HttpResponse::BadRequest()
            .json(json!({"code":400,"message":"Validation error","errors":e}));
    }
    // 1. Check for Existing User (Async I/O)
    // Main Thread / Async
    let user_check = Entity::find()
        .filter(
            Condition::any()
                .add(Column::Username.eq(&form.username))
                .add(Column::Email.eq(&form.email)),
        )
        .one(db.get_ref())
        .await;

    match user_check {
        Ok(Some(res)) => {
            if res.username == form.username {
                warn!(
                    "Registration failed: username {} already exists",
                    form.username
                );
                return HttpResponse::Conflict()
                    .json(json!({"code":409,"message":"username already exists"}));
            }
            if res.email == form.email {
                warn!("Registration failed: email {} already exists", form.email);
                return HttpResponse::Conflict()
                    .json(json!({"code":409,"message":"email already exists"}));
            } else {
                warn!("Registration failed: user/email already exists");
                return HttpResponse::Conflict()
                    .json(json!({"code":409,"message":"user/email already exists"}));
            }
        }
        Ok(None) => (), // No duplicate found, proceed.
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"code":500,"message":"Internal server error"}));
        }
    };

    let password_input = form.password.clone();

    // 2. CPU Intensive Task (Argon2 Hashing)
    // Hashing is computationally expensive by design (to prevent brute-force).
    // Offloading to `web::block` ensures the server remains responsive to other requests.
    // Blocking Thread / Sync
    let hash_result = web::block(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(password_input.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(format!("Password hashing error: {}", e)),
        }
    })
    .await;

    // 3. Handle Hash Result
    let password_hash = match hash_result {
        // Outer Ok + Inner Ok: Hashing succeeded.
        Ok(Ok(hash)) => hash,

        // Outer Ok + Inner Err: Argon2 failed to hash (e.g., internal library error).
        Ok(Err(e)) => {
            error!("Hashing logic error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"code":500,"message":"Internal server error"}));
        }

        // Outer Err: Thread pool execution failed.
        Err(e) => {
            error!("Blocking execution error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"code":500,"message":"Internal server error"}));
        }
    };

    // 4. Insert New User (Async I/O)
    let form_data = form.into_inner();
    let create_user: ActiveModel = (form_data, password_hash).into();

    match Entity::insert(create_user).exec(db.get_ref()).await {
        Ok(res) => {
            info!("New user registered with ID: {}", res.last_insert_id);
            HttpResponse::Created().json(json!({"code":201,"message":"User registered successfully","user_id":res.last_insert_id}))
        }
        Err(db_err) => match db_err.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(msg)) => {
                warn!(
                    "Registration failed due to unique constraint violation: {}",
                    msg
                );
                HttpResponse::Conflict()
                    .json(json!({"code":409,"message":"User with provided details already exists"}))
            }
            _ => {
                error!("Database insertion error: {}", db_err);
                HttpResponse::InternalServerError()
                    .json(json!({"code":500,"message":"Internal server error"}))
            }
        },
    }
}
