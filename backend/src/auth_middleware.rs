//! JWT bearer-token extractor used to protect account and metered endpoints.
//!
//! The login handler in `auth.rs` issues HS256 tokens carrying `Claims`.
//! `AuthUser` validates the `Authorization: Bearer <token>` header against the
//! configured `jwt_secret` and exposes the authenticated identity to handlers.

use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};

use crate::auth::Claims;
use crate::AppState;

pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(extract(req))
    }
}

fn extract(req: &HttpRequest) -> Result<AuthUser, Error> {
    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorUnauthorized("server misconfiguration"))?;

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ErrorUnauthorized("missing bearer token"))?;

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ErrorUnauthorized("invalid or expired token"))?;

    Ok(AuthUser {
        id: data.claims.sub,
        email: data.claims.email,
        name: data.claims.name,
    })
}
