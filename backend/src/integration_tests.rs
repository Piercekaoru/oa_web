//! DB-backed integration tests for the credit/payment lifecycle.
//!
//! They run only when `TEST_DATABASE_URL` points at a throwaway Postgres (e.g.
//! `postgres://kayano@localhost:5432/openachieve_test`); without it each test
//! no-ops so `cargo test` stays green on machines with no database. Run with
//! `--test-threads=1` so the idempotent schema setup doesn't race.

use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

use crate::{credits, db, plans};

/// Connect to the test database, or return None (skip) if it isn't configured.
async fn connect() -> Option<Client> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;
    let (client, connection) = match tokio_postgres::connect(&url, NoTls).await {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("skipping DB test: cannot connect to TEST_DATABASE_URL: {e}");
            return None;
        }
    };
    tokio::spawn(async move {
        let _ = connection.await;
    });
    db::init_db(&client).await.expect("init_db");
    Some(client)
}

/// Insert a fresh, verified user with a unique email. Returns the user id.
async fn make_user(client: &Client) -> String {
    let id = Uuid::new_v4().to_string();
    let email = format!("it-{id}@example.com");
    client
        .execute(
            "INSERT INTO users (id, email, name, password_hash, verified)
             VALUES ($1, $2, 'IT User', 'placeholder-hash', TRUE)",
            &[&id, &email],
        )
        .await
        .expect("insert user");
    id
}

async fn email_of(client: &Client, user_id: &str) -> String {
    client
        .query_one("SELECT email FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap()
        .get(0)
}

#[tokio::test]
async fn pro_then_upgrade_to_max_adds_delta_and_keeps_period() {
    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;

    credits::grant_plan(&c, &uid, "pro").await.unwrap();
    let sub = db::get_subscription(&c, &uid).await.unwrap().unwrap();
    assert_eq!(sub.plan, "pro");
    assert_eq!(sub.credits_remaining, 10_000);
    let period_before = sub.period_end.clone();

    // The synthetic "max_upgrade" order tops up the Max−Pro delta in place.
    credits::grant_plan(&c, &uid, "max_upgrade").await.unwrap();
    let sub = db::get_subscription(&c, &uid).await.unwrap().unwrap();
    assert_eq!(sub.plan, "max");
    assert_eq!(sub.credits_remaining, 20_000); // 10k Pro + 10k delta
    assert_eq!(sub.period_end, period_before, "upgrade must not reset the period");
}

#[tokio::test]
async fn upgrade_without_active_sub_returns_false_and_falls_back() {
    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;

    // No subscription yet -> nothing to upgrade in place.
    let upgraded = db::upgrade_to_max(&c, &uid, 10_000).await.unwrap();
    assert!(!upgraded, "no active subscription to upgrade");

    // grant_plan's fallback still grants a Max period worth the delta.
    credits::grant_plan(&c, &uid, "max_upgrade").await.unwrap();
    let sub = db::get_subscription(&c, &uid).await.unwrap().unwrap();
    assert_eq!(sub.plan, "max");
    assert_eq!(sub.credits_remaining, 10_000);
}

#[tokio::test]
async fn mark_order_paid_is_idempotent() {
    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;
    let order = db::create_payment_order(&c, &uid, "pro", 10_000, "fovpay")
        .await
        .unwrap();

    assert!(
        db::mark_order_paid(&c, &order, "prov-1").await.unwrap(),
        "first callback flips pending -> paid"
    );
    assert!(
        !db::mark_order_paid(&c, &order, "prov-1").await.unwrap(),
        "duplicate callback is a no-op (no double grant)"
    );
}

#[tokio::test]
async fn reset_password_accepts_correct_code_once() {
    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;
    let email = email_of(&c, &uid).await;

    db::set_reset_code(&c, &email, "123456").await.unwrap();

    // Wrong code is rejected.
    assert!(!db::reset_password(&c, &email, "000000", "newhash").await.unwrap());

    // Correct code succeeds, updates the hash, clears the code, marks verified.
    assert!(db::reset_password(&c, &email, "123456", "newhash").await.unwrap());
    let row = c
        .query_one(
            "SELECT password_hash, reset_code, verified FROM users WHERE email = $1",
            &[&email],
        )
        .await
        .unwrap();
    let hash: String = row.get(0);
    let code: Option<String> = row.get(1);
    let verified: bool = row.get(2);
    assert_eq!(hash, "newhash");
    assert!(code.is_none(), "reset code is cleared after use");
    assert!(verified, "a successful reset verifies the account");

    // The code is single-use.
    assert!(!db::reset_password(&c, &email, "123456", "other").await.unwrap());
}

#[tokio::test]
async fn reset_password_rejects_expired_code() {
    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;
    let email = email_of(&c, &uid).await;

    db::set_reset_code(&c, &email, "654321").await.unwrap();
    c.execute(
        "UPDATE users SET reset_expires_at = NOW() - INTERVAL '1 minute' WHERE email = $1",
        &[&email],
    )
    .await
    .unwrap();

    assert!(
        !db::reset_password(&c, &email, "654321", "newhash").await.unwrap(),
        "an expired code must be rejected"
    );
}

#[tokio::test]
async fn active_max_outranks_lower_tiers() {
    // The checkout gate rejects a purchase whose tier rank <= the active plan's.
    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;
    credits::grant_plan(&c, &uid, "max").await.unwrap();

    let sub = credits::current(&c, &uid).await.unwrap().unwrap();
    assert_eq!(sub.status, "active");
    assert_eq!(sub.plan, "max");
    assert!(plans::rank("plus") <= plans::rank(&sub.plan));
    assert!(plans::rank("pro") <= plans::rank(&sub.plan));
}

#[actix_web::test]
async fn checkout_rejects_downgrade_for_active_max() {
    use actix_web::{http::StatusCode, test, web, App};

    let Some(c) = connect().await else { return };
    let uid = make_user(&c).await;
    let email = email_of(&c, &uid).await;
    credits::grant_plan(&c, &uid, "max").await.unwrap();

    let jwt_secret = "test-secret".to_string();
    let token = crate::auth::issue_jwt(&jwt_secret, &uid, &email, "IT User", 7).unwrap();

    let state = crate::AppState {
        db: c,
        jwt_secret,
        resend_api_key: String::new(),
        resend_from: String::new(),
        openrouter_api_key: String::new(),
        cpa_base_url: String::new(),
        cpa_api_key: String::new(),
        fovpay_pid: String::new(),
        fovpay_key: String::new(),
        frontend_base_url: "http://localhost".into(),
        public_base_url: "http://localhost".into(),
        usd_cny_rate: 7.2,
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/api/checkout", web::post().to(crate::api::checkout)),
    )
    .await;

    // A Max user buying Plus must be rejected before any payment-provider call.
    let req = test::TestRequest::post()
        .uri("/api/checkout")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(serde_json::json!({ "plan": "plus" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
