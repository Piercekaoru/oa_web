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
    let password_matches = bcrypt::verify(password, &user.password_hash).unwrap_or(false);
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
