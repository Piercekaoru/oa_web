use actix_web::{web, HttpResponse};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::email;
use crate::AppState;

// ─── Request / Response types ───

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub email: String,
    pub code: String,
}

#[derive(Deserialize)]
pub struct ForgotRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetRequest {
    pub email: String,
    pub code: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct GoogleRequest {
    pub credential: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // user id
    pub email: String,
    pub name: String,
    pub exp: usize,      // expiration timestamp
}

// ─── Token helper ───

/// Issue an HS256 JWT carrying `Claims`, valid for `days`. Shared by the login
/// handler and the device-authorization flow; validated by `AuthUser`.
pub fn issue_jwt(
    secret: &str,
    id: &str,
    email: &str,
    name: &str,
    days: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(days))
        .expect("valid timestamp")
        .timestamp() as usize;
    let claims = Claims {
        sub: id.to_string(),
        email: email.to_string(),
        name: name.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

// ─── Handlers ───

/// POST /api/register
pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let name = body.name.trim().to_string();
    let password = &body.password;

    // Basic validation
    if email.is_empty() || password.len() < 6 {
        return HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: "Email and password (min 6 chars) are required.".into(),
            token: None,
            user: None,
        });
    }

    // Check if user already exists
    match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(AuthResponse {
                success: false,
                message: "An account with this email already exists.".into(),
                token: None,
                user: None,
            });
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
        _ => {}
    }

    // Hash password
    let password_hash = match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("bcrypt error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
    };

    // Generate 6-digit verification code
    let code = format!("{:06}", rand::random::<u32>() % 1_000_000);

    // Create user in DB
    match db::create_user(&state.db, &email, &name, &password_hash, &code).await {
        Ok(_id) => {}
        Err(e) => {
            eprintln!("DB insert error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Failed to create account.".into(),
                token: None,
                user: None,
            });
        }
    }

    // Attempt to send verification email (non-blocking, best-effort)
    if !state.resend_api_key.is_empty() {
        let _ = email::send_verification_email(
            &state.resend_api_key,
            &state.resend_from,
            &email,
            &code,
        )
        .await;
    } else {
        println!("📧 Verification code for {}: {} (email sending disabled)", email, code);
    }

    HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "Account created. Please check your email for the verification code.".into(),
        token: None,
        user: None,
    })
}

/// POST /api/verify
pub async fn verify(
    state: web::Data<AppState>,
    body: web::Json<VerifyRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let code = body.code.trim().to_string();

    let user = match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound().json(AuthResponse {
                success: false,
                message: "Account not found.".into(),
                token: None,
                user: None,
            });
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
    };

    if user.verified {
        return HttpResponse::Ok().json(AuthResponse {
            success: true,
            message: "Account is already verified.".into(),
            token: None,
            user: None,
        });
    }

    match &user.verification_code {
        Some(stored_code) if stored_code == &code => {
            let _ = db::verify_user(&state.db, &email).await;
            HttpResponse::Ok().json(AuthResponse {
                success: true,
                message: "Account verified successfully! You can now log in.".into(),
                token: None,
                user: None,
            })
        }
        _ => HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: "Invalid verification code.".into(),
            token: None,
            user: None,
        }),
    }
}

/// POST /api/login
pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let password = &body.password;

    let user = match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "Invalid email or password.".into(),
                token: None,
                user: None,
            });
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
    };

    // Verify password
    let password_matches = match &user.password_hash {
        Some(hash) => bcrypt::verify(password, hash).unwrap_or(false),
        None => {
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "This account uses Google sign-in.".into(),
                token: None,
                user: None,
            });
        }
    };
    if !password_matches {
        return HttpResponse::Unauthorized().json(AuthResponse {
            success: false,
            message: "Invalid email or password.".into(),
            token: None,
            user: None,
        });
    }

    // Check verified status
    if !user.verified {
        return HttpResponse::Forbidden().json(AuthResponse {
            success: false,
            message: "Please verify your email before logging in.".into(),
            token: None,
            user: None,
        });
    }

    // Generate JWT (valid for 7 days)
    let token = match issue_jwt(&state.jwt_secret, &user.id, &user.email, &user.name, 7) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Failed to generate token.".into(),
                token: None,
                user: None,
            });
        }
    };

    HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "Login successful.".into(),
        token: Some(token),
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
        }),
    })
}

/// POST /api/forgot-password — email a 6-digit reset code. Always returns the
/// same success response so it can't be used to probe which emails are registered.
pub async fn forgot_password(
    state: web::Data<AppState>,
    body: web::Json<ForgotRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();

    match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(_)) => {
            let code = format!("{:06}", rand::random::<u32>() % 1_000_000);
            if let Err(e) = db::set_reset_code(&state.db, &email, &code).await {
                eprintln!("DB error: {}", e);
                return HttpResponse::InternalServerError().json(AuthResponse {
                    success: false,
                    message: "Internal server error.".into(),
                    token: None,
                    user: None,
                });
            }
            if !state.resend_api_key.is_empty() {
                let _ = email::send_password_reset_email(
                    &state.resend_api_key,
                    &state.resend_from,
                    &email,
                    &code,
                )
                .await;
            } else {
                println!("📧 Password reset code for {}: {} (email sending disabled)", email, code);
            }
        }
        Ok(None) => {} // Do not reveal whether the account exists.
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
    }

    HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "If an account with that email exists, we've sent a reset code.".into(),
        token: None,
        user: None,
    })
}

/// POST /api/reset-password — set a new password using a valid, unexpired code.
pub async fn reset_password(
    state: web::Data<AppState>,
    body: web::Json<ResetRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let code = body.code.trim().to_string();
    let password = &body.password;

    if password.len() < 6 {
        return HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: "Password must be at least 6 characters.".into(),
            token: None,
            user: None,
        });
    }

    let password_hash = match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("bcrypt error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
    };

    match db::reset_password(&state.db, &email, &code, &password_hash).await {
        Ok(true) => HttpResponse::Ok().json(AuthResponse {
            success: true,
            message: "Password reset. Please log in.".into(),
            token: None,
            user: None,
        }),
        Ok(false) => HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: "Invalid or expired reset code.".into(),
            token: None,
            user: None,
        }),
        Err(e) => {
            eprintln!("DB error: {}", e);
            HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            })
        }
    }
}

fn google_unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().json(AuthResponse {
        success: false,
        message: "Google sign-in failed.".into(),
        token: None,
        user: None,
    })
}

/// Validate a Google `tokeninfo` response against our client id and extract the
/// identity. Pure (no I/O) so it can be unit-tested. `email_verified` may be a
/// JSON string ("true") or a bool depending on the endpoint.
pub fn parse_google_claims(
    body: &serde_json::Value,
    expected_aud: &str,
) -> Option<(String, String, String)> {
    if body.get("aud").and_then(|v| v.as_str())? != expected_aud {
        return None;
    }
    let email_verified = match body.get("email_verified") {
        Some(serde_json::Value::String(s)) => s == "true",
        Some(serde_json::Value::Bool(b)) => *b,
        _ => false,
    };
    if !email_verified {
        return None;
    }
    let email = body.get("email").and_then(|v| v.as_str())?.trim().to_lowercase();
    if email.is_empty() {
        return None;
    }
    let sub = body.get("sub").and_then(|v| v.as_str())?.to_string();
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());
    Some((email, name, sub))
}

/// POST /api/auth/google — verify a Google ID token (credential) and sign in,
/// creating or linking the account by its verified email.
pub async fn google_login(
    state: web::Data<AppState>,
    body: web::Json<GoogleRequest>,
) -> HttpResponse {
    if state.google_client_id.is_empty() {
        return HttpResponse::ServiceUnavailable().json(AuthResponse {
            success: false,
            message: "Google sign-in is not configured.".into(),
            token: None,
            user: None,
        });
    }

    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        body.credential
    );
    let resp = match reqwest::Client::new().get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(_) => return google_unauthorized(),
        Err(e) => {
            eprintln!("google tokeninfo error: {}", e);
            return google_unauthorized();
        }
    };
    let claims: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return google_unauthorized(),
    };
    let (email, name, sub) = match parse_google_claims(&claims, &state.google_client_id) {
        Some(t) => t,
        None => return google_unauthorized(),
    };

    let user = match db::find_or_create_google_user(&state.db, &email, &name, &sub).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Internal server error.".into(),
                token: None,
                user: None,
            });
        }
    };

    let token = match issue_jwt(&state.jwt_secret, &user.id, &user.email, &user.name, 7) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {}", e);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Failed to generate token.".into(),
                token: None,
                user: None,
            });
        }
    };

    HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "Login successful.".into(),
        token: Some(token),
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_google_claims_accepts_valid_token() {
        let body = json!({
            "aud": "client-123",
            "email": "User@Example.com",
            "email_verified": "true",
            "sub": "g-sub-1",
            "name": "User"
        });
        let (email, name, sub) = parse_google_claims(&body, "client-123").unwrap();
        assert_eq!(email, "user@example.com"); // normalized to lowercase
        assert_eq!(name, "User");
        assert_eq!(sub, "g-sub-1");
    }

    #[test]
    fn parse_google_claims_rejects_wrong_aud() {
        let body = json!({ "aud": "someone-else", "email": "a@b.com", "email_verified": "true", "sub": "x" });
        assert!(parse_google_claims(&body, "client-123").is_none());
    }

    #[test]
    fn parse_google_claims_rejects_unverified_email() {
        let body = json!({ "aud": "client-123", "email": "a@b.com", "email_verified": "false", "sub": "x" });
        assert!(parse_google_claims(&body, "client-123").is_none());
    }

    #[test]
    fn parse_google_claims_requires_email() {
        let body = json!({ "aud": "client-123", "email_verified": true, "sub": "x" });
        assert!(parse_google_claims(&body, "client-123").is_none());
    }

    #[test]
    fn parse_google_claims_defaults_name_to_email_local_part() {
        let body = json!({ "aud": "client-123", "email": "alice@example.com", "email_verified": true, "sub": "x" });
        let (_, name, _) = parse_google_claims(&body, "client-123").unwrap();
        assert_eq!(name, "alice");
    }
}
