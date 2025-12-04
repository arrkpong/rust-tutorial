// src/utils/auth_middleware.rs
use crate::utils::jwt::decode_jwt;
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, error::ErrorUnauthorized};
use futures::future::{Ready, ready};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => return ready(Err(ErrorUnauthorized("Authorization header missing"))),
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => return ready(Err(ErrorUnauthorized("Invalid Authorization header"))),
        };

        if !auth_str.starts_with("Bearer ") {
            return ready(Err(ErrorUnauthorized("Invalid Authorization scheme")));
        }

        let token = &auth_str[7..]; // Skip "Bearer "
        match decode_jwt(token) {
            Ok(claims) => {
                let user = AuthenticatedUser {
                    username: claims.sub,
                };
                ready(Ok(user))
            }
            Err(_) => ready(Err(ErrorUnauthorized("Invalid or expired token"))),
        }
    }
}
