//! OAuth 2.0 device-authorization flow for the `oa` CLI.
//!
//! The CLI requests a device + user code, shows the user a verification URL, and
//! polls the token endpoint. The user approves from the web dashboard (which
//! calls `approve` with their session JWT). An approved device code mints a
//! normal access JWT (validated by `auth_middleware::AuthUser`) plus a refresh
//! token, so the metered proxy needs no special handling for CLI requests.

use actix_web::{web, HttpResponse};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::auth::issue_jwt;
use crate::auth_middleware::AuthUser;
use crate::{db, AppState};

const DEVICE_CODE_TTL_SECS: i32 = 900;
const ACCESS_TOKEN_DAYS: i64 = 7;
const REFRESH_TOKEN_DAYS: i32 = 90;
const POLL_INTERVAL_SECS: i64 = 5;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn random_hex(bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..bytes).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
}

/// 8-char user code (no ambiguous chars) grouped as XXXX-XXXX.
fn random_user_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(9);
    for i in 0..8 {
        if i == 4 {
            s.push('-');
        }
        let idx = rng.gen_range(0..ALPHABET.len());
        s.push(ALPHABET[idx] as char);
    }
    s
}

fn bad(error: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "error": error }))
}

fn server_error() -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "error": "server_error" }))
}

/// POST /api/auth/device/code — start a device-authorization flow.
pub async fn device_code(state: web::Data<AppState>) -> HttpResponse {
    let device_code = random_hex(32);
    let user_code = random_user_code();

    if let Err(e) = db::create_device_code(&state.db, &device_code, &user_code, DEVICE_CODE_TTL_SECS).await {
        eprintln!("DB error: {}", e);
        return server_error();
    }

    let base = state.frontend_base_url.trim_end_matches('/');
    HttpResponse::Ok().json(json!({
        "device_code": device_code,
        "user_code": user_code,
        "verification_uri": format!("{}/#device", base),
        "verification_uri_complete": format!("{}/#device?code={}", base, user_code),
        "interval": POLL_INTERVAL_SECS,
        "expires_in": DEVICE_CODE_TTL_SECS,
    }))
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    #[serde(default)]
    pub device_code: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// POST /api/auth/token — device_code and refresh_token grants.
pub async fn token(state: web::Data<AppState>, body: web::Json<TokenRequest>) -> HttpResponse {
    match body.grant_type.as_str() {
        "device_code" => {
            let Some(code) = body.device_code.clone() else {
                return bad("invalid_request");
            };
            let row = match db::get_device_code(&state.db, &code).await {
                Ok(Some(r)) => r,
                Ok(None) => return bad("invalid_grant"),
                Err(e) => {
                    eprintln!("DB error: {}", e);
                    return server_error();
                }
            };
            if row.expired {
                return bad("expired_token");
            }
            match row.status.as_str() {
                "pending" => return bad("authorization_pending"),
                "denied" => return bad("access_denied"),
                "approved" => {}
                _ => return bad("invalid_grant"),
            }
            let Some(user_id) = row.user_id else {
                return bad("invalid_grant");
            };
            let user = match db::find_user_by_id(&state.db, &user_id).await {
                Ok(Some(u)) => u,
                _ => return bad("invalid_grant"),
            };

            let access = match issue_jwt(&state.jwt_secret, &user.id, &user.email, &user.name, ACCESS_TOKEN_DAYS) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("JWT error: {}", e);
                    return server_error();
                }
            };
            let refresh = random_hex(32);
            if let Err(e) = db::store_refresh_token(&state.db, &user.id, &hash_token(&refresh), REFRESH_TOKEN_DAYS).await {
                eprintln!("DB error: {}", e);
                return server_error();
            }
            let _ = db::claim_device_code(&state.db, &code).await;

            HttpResponse::Ok().json(json!({
                "access_token": access,
                "refresh_token": refresh,
                "token_type": "Bearer",
                "expires_in": ACCESS_TOKEN_DAYS * 24 * 3600,
            }))
        }
        "refresh_token" => {
            let Some(refresh) = body.refresh_token.clone() else {
                return bad("invalid_request");
            };
            let row = match db::get_refresh_token(&state.db, &hash_token(&refresh)).await {
                Ok(Some(r)) => r,
                Ok(None) => return bad("invalid_grant"),
                Err(e) => {
                    eprintln!("DB error: {}", e);
                    return server_error();
                }
            };
            if row.revoked || row.expired {
                return bad("invalid_grant");
            }
            let user = match db::find_user_by_id(&state.db, &row.user_id).await {
                Ok(Some(u)) => u,
                _ => return bad("invalid_grant"),
            };
            let access = match issue_jwt(&state.jwt_secret, &user.id, &user.email, &user.name, ACCESS_TOKEN_DAYS) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("JWT error: {}", e);
                    return server_error();
                }
            };
            HttpResponse::Ok().json(json!({
                "access_token": access,
                "refresh_token": refresh,
                "token_type": "Bearer",
                "expires_in": ACCESS_TOKEN_DAYS * 24 * 3600,
            }))
        }
        _ => bad("unsupported_grant_type"),
    }
}

#[derive(Deserialize)]
pub struct ApproveRequest {
    pub user_code: String,
}

/// POST /api/auth/device/approve — the logged-in user authorizes a device code.
pub async fn approve(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<ApproveRequest>,
) -> HttpResponse {
    let code = body.user_code.trim().to_uppercase();
    match db::approve_device_code(&state.db, &code, &user.id).await {
        Ok(true) => HttpResponse::Ok().json(json!({ "success": true })),
        Ok(false) => HttpResponse::BadRequest()
            .json(json!({ "success": false, "message": "Invalid or expired code." })),
        Err(e) => {
            eprintln!("DB error: {}", e);
            HttpResponse::InternalServerError().json(json!({ "success": false }))
        }
    }
}
