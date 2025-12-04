// src/utils/jwt.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn encode_jwt(username: String) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = Utc::now();
    let exp = now + Duration::hours(24); // Token valid for 24 hours
    let claims = Claims {
        sub: username,
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };
    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn decode_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60; // 1 minute leeway
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;
    Ok(token_data.claims)
}
