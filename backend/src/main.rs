mod api;
mod auth;
mod auth_device;
mod auth_middleware;
mod cpa;
mod credits;
mod db;
mod email;
mod openrouter;
mod payment;
mod plans;

#[cfg(test)]
mod integration_tests;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tokio_postgres::NoTls;

pub struct AppState {
    pub db: tokio_postgres::Client,
    pub jwt_secret: String,
    pub resend_api_key: String,
    pub resend_from: String,
    pub openrouter_api_key: String,
    pub cpa_base_url: String,
    pub cpa_api_key: String,
    pub fovpay_pid: String,
    pub fovpay_key: String,
    pub frontend_base_url: String,
    pub public_base_url: String,
    pub usd_cny_rate: f64,
    pub google_client_id: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://oa:oa_secret@localhost:5432/openachieve".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "oa-jwt-secret-key-2026".to_string());
    let resend_api_key = std::env::var("RESEND_API_KEY").unwrap_or_default();
    let resend_from = std::env::var("RESEND_FROM")
        .unwrap_or_else(|_| "onboarding@resend.dev".to_string());
    let openrouter_api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
    let cpa_base_url = std::env::var("CPA_BASE_URL").unwrap_or_default();
    let cpa_api_key = std::env::var("CPA_API_KEY").unwrap_or_default();
    let fovpay_pid = std::env::var("FOVPAY_PID").unwrap_or_default();
    let fovpay_key = std::env::var("FOVPAY_KEY").unwrap_or_default();
    let frontend_base_url =
        std::env::var("FRONTEND_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let public_base_url =
        std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let usd_cny_rate = std::env::var("FOVPAY_USD_CNY_RATE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(7.20);
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();

    // Connect to PostgreSQL with retry logic
    let client = connect_with_retry(&database_url, 10).await;

    // Initialize database schema and seed data
    db::init_db(&client).await.expect("Failed to initialize database");
    db::seed_test_user(&client).await.expect("Failed to seed test user");

    println!("✅ Database initialized and test user seeded.");
    println!("🚀 Starting server at http://0.0.0.0:8080");

    let state = web::Data::new(AppState {
        db: client,
        jwt_secret,
        resend_api_key,
        resend_from,
        openrouter_api_key,
        cpa_base_url,
        cpa_api_key,
        fovpay_pid,
        fovpay_key,
        frontend_base_url,
        public_base_url,
        usd_cny_rate,
        google_client_id,
    });

    let cors_origins: Vec<String> = match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(s) if !s.trim().is_empty() => s
            .split(',')
            .map(|o| o.trim().to_string())
            .filter(|o| !o.is_empty())
            .collect(),
        _ => vec![
            "http://localhost:5173".to_string(),
            "http://localhost:3000".to_string(),
            "http://127.0.0.1:5173".to_string(),
            "http://127.0.0.1:3000".to_string(),
        ],
    };

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .supports_credentials()
            .max_age(3600);
        for origin in &cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route("/api/register", web::post().to(auth::register))
            .route("/api/verify", web::post().to(auth::verify))
            .route("/api/login", web::post().to(auth::login))
            .route("/api/auth/google", web::post().to(auth::google_login))
            .route("/api/forgot-password", web::post().to(auth::forgot_password))
            .route("/api/reset-password", web::post().to(auth::reset_password))
            .route("/api/auth/device/code", web::post().to(auth_device::device_code))
            .route("/api/auth/device/approve", web::post().to(auth_device::approve))
            .route("/api/auth/token", web::post().to(auth_device::token))
            .route("/api/me", web::get().to(api::me))
            .route("/api/checkout", web::post().to(api::checkout))
            .route("/api/orders/{id}", web::get().to(api::order_status))
            .route("/api/webhooks/payment", web::post().to(api::payment_webhook))
            .route("/api/v1/models", web::get().to(api::models))
            .route("/api/v1/chat/completions", web::post().to(api::chat_completions))
            .route("/api/health", web::get().to(|| async {
                actix_web::HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn connect_with_retry(database_url: &str, max_retries: u32) -> tokio_postgres::Client {
    for i in 0..max_retries {
        match tokio_postgres::connect(database_url, NoTls).await {
            Ok((client, connection)) => {
                // Spawn the connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("PostgreSQL connection error: {}", e);
                    }
                });
                println!("✅ Connected to PostgreSQL");
                return client;
            }
            Err(e) => {
                eprintln!(
                    "⏳ Failed to connect to PostgreSQL (attempt {}/{}): {}",
                    i + 1,
                    max_retries,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
    panic!("❌ Could not connect to PostgreSQL after {} retries", max_retries);
}
