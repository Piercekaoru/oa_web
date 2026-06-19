use tokio_postgres::Client;
use uuid::Uuid;

/// Initialize the database schema.
pub async fn init_db(client: &Client) -> Result<(), tokio_postgres::Error> {
    client
        .batch_execute(
            "
            CREATE TABLE IF NOT EXISTS users (
                id          TEXT PRIMARY KEY,
                email       TEXT UNIQUE NOT NULL,
                name        TEXT NOT NULL DEFAULT '',
                password_hash TEXT NOT NULL,
                verified    BOOLEAN NOT NULL DEFAULT FALSE,
                verification_code TEXT,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS subscriptions (
                id                TEXT PRIMARY KEY,
                user_id           TEXT NOT NULL REFERENCES users(id),
                plan              TEXT NOT NULL,
                status            TEXT NOT NULL,
                credits_remaining INTEGER NOT NULL DEFAULT 0,
                credits_granted   INTEGER NOT NULL DEFAULT 0,
                period_start      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                period_end        TIMESTAMPTZ NOT NULL,
                created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE UNIQUE INDEX IF NOT EXISTS subscriptions_user_id_uniq ON subscriptions(user_id);

            CREATE TABLE IF NOT EXISTS credit_ledger (
                id            TEXT PRIMARY KEY,
                user_id       TEXT NOT NULL REFERENCES users(id),
                delta         INTEGER NOT NULL,
                reason        TEXT NOT NULL,
                balance_after INTEGER NOT NULL,
                usage_id      TEXT,
                created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS usage_events (
                id                       TEXT PRIMARY KEY,
                user_id                  TEXT NOT NULL REFERENCES users(id),
                model                    TEXT NOT NULL,
                prompt_tokens            INTEGER NOT NULL DEFAULT 0,
                completion_tokens        INTEGER NOT NULL DEFAULT 0,
                openrouter_cost_micros   BIGINT NOT NULL,
                credits_charged          INTEGER NOT NULL,
                openrouter_generation_id TEXT,
                created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS payment_orders (
                id                TEXT PRIMARY KEY,
                user_id           TEXT NOT NULL REFERENCES users(id),
                plan              TEXT NOT NULL,
                amount_cents      INTEGER NOT NULL,
                status            TEXT NOT NULL,
                provider          TEXT NOT NULL,
                provider_order_id TEXT UNIQUE,
                created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS device_codes (
                device_code TEXT PRIMARY KEY,
                user_code   TEXT UNIQUE NOT NULL,
                user_id     TEXT REFERENCES users(id),
                status      TEXT NOT NULL DEFAULT 'pending',
                expires_at  TIMESTAMPTZ NOT NULL,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS refresh_tokens (
                id         TEXT PRIMARY KEY,
                user_id    TEXT NOT NULL REFERENCES users(id),
                token_hash TEXT UNIQUE NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                revoked    BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            ",
        )
        .await?;

    println!("✅ Database schema ready.");
    Ok(())
}

/// Seed the pre-configured test user if it doesn't already exist.
pub async fn seed_test_user(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let email = "1250585873@qq.com";

    // Check if the test user already exists
    let row = client
        .query_opt("SELECT id FROM users WHERE email = $1", &[&email])
        .await?;

    if row.is_some() {
        println!("ℹ️  Test user already exists, skipping seed.");
        return Ok(());
    }

    let id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash("12138Wsx.", bcrypt::DEFAULT_COST)
        .map_err(|e| format!("bcrypt error: {}", e))?;

    client
        .execute(
            "INSERT INTO users (id, email, name, password_hash, verified) VALUES ($1, $2, $3, $4, $5)",
            &[&id, &email, &"Test User", &password_hash, &true],
        )
        .await?;

    println!("✅ Test user seeded: {}", email);
    Ok(())
}

/// Insert a new user into the database. Returns the new user's ID.
pub async fn create_user(
    client: &Client,
    email: &str,
    name: &str,
    password_hash: &str,
    verification_code: &str,
) -> Result<String, tokio_postgres::Error> {
    let id = Uuid::new_v4().to_string();

    client
        .execute(
            "INSERT INTO users (id, email, name, password_hash, verified, verification_code) VALUES ($1, $2, $3, $4, FALSE, $5)",
            &[&id, &email, &name, &password_hash, &verification_code],
        )
        .await?;

    Ok(id)
}

/// A lightweight struct representing a user row.
#[derive(Debug)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub verified: bool,
    pub verification_code: Option<String>,
}

/// Find a user by email.
pub async fn find_user_by_email(
    client: &Client,
    email: &str,
) -> Result<Option<UserRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT id, email, name, password_hash, verified, verification_code FROM users WHERE email = $1",
            &[&email],
        )
        .await?;

    Ok(row.map(|r| UserRow {
        id: r.get(0),
        email: r.get(1),
        name: r.get(2),
        password_hash: r.get(3),
        verified: r.get(4),
        verification_code: r.get(5),
    }))
}

/// Mark a user as verified and clear the verification code.
pub async fn verify_user(client: &Client, email: &str) -> Result<u64, tokio_postgres::Error> {
    let rows_affected = client
        .execute(
            "UPDATE users SET verified = TRUE, verification_code = NULL WHERE email = $1",
            &[&email],
        )
        .await?;

    Ok(rows_affected)
}

// ─── Subscriptions & credits ───

#[derive(Debug)]
pub struct SubscriptionRow {
    pub plan: String,
    pub status: String,
    pub credits_remaining: i32,
    pub period_end: String,
    pub expired: bool,
}

/// Fetch a user's subscription, if any. `expired` is computed against NOW().
pub async fn get_subscription(
    client: &Client,
    user_id: &str,
) -> Result<Option<SubscriptionRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT plan, status, credits_remaining,
                    to_char(period_end, 'YYYY-MM-DD') AS period_end,
                    period_end < NOW() AS expired
             FROM subscriptions WHERE user_id = $1",
            &[&user_id],
        )
        .await?;

    Ok(row.map(|r| SubscriptionRow {
        plan: r.get(0),
        status: r.get(1),
        credits_remaining: r.get(2),
        period_end: r.get(3),
        expired: r.get(4),
    }))
}

/// Grant (or reset) a 30-day plan period with a fresh credit allotment.
/// Credits do not roll over: remaining is overwritten with the new grant.
pub async fn grant_period(
    client: &Client,
    user_id: &str,
    plan: &str,
    credits: i32,
) -> Result<(), tokio_postgres::Error> {
    let sub_id = Uuid::new_v4().to_string();
    client
        .execute(
            "INSERT INTO subscriptions
                (id, user_id, plan, status, credits_remaining, credits_granted, period_start, period_end)
             VALUES ($1, $2, $3, 'active', $4, $4, NOW(), NOW() + INTERVAL '30 days')
             ON CONFLICT (user_id) DO UPDATE SET
                plan = EXCLUDED.plan,
                status = 'active',
                credits_remaining = EXCLUDED.credits_remaining,
                credits_granted = EXCLUDED.credits_granted,
                period_start = NOW(),
                period_end = NOW() + INTERVAL '30 days',
                updated_at = NOW()",
            &[&sub_id, &user_id, &plan, &credits],
        )
        .await?;

    let ledger_id = Uuid::new_v4().to_string();
    client
        .execute(
            "INSERT INTO credit_ledger (id, user_id, delta, reason, balance_after)
             VALUES ($1, $2, $3, 'grant', $3)",
            &[&ledger_id, &user_id, &credits],
        )
        .await?;

    Ok(())
}

/// Upgrade an active subscription to Max in place: add `delta_credits` and flip
/// the plan to "max" while keeping the current period. Returns false if the user
/// has no active subscription to upgrade (caller decides on a fallback).
pub async fn upgrade_to_max(
    client: &Client,
    user_id: &str,
    delta_credits: i32,
) -> Result<bool, tokio_postgres::Error> {
    let ledger_id = Uuid::new_v4().to_string();
    let row = client
        .query_opt(
            "WITH upd AS (
                UPDATE subscriptions
                SET plan = 'max',
                    credits_remaining = credits_remaining + $2,
                    credits_granted = credits_granted + $2,
                    updated_at = NOW()
                WHERE user_id = $1 AND status = 'active'
                RETURNING credits_remaining
             )
             INSERT INTO credit_ledger (id, user_id, delta, reason, balance_after)
             SELECT $3, $1, $2, 'upgrade', credits_remaining FROM upd
             RETURNING balance_after",
            &[&user_id, &delta_credits, &ledger_id],
        )
        .await?;
    Ok(row.is_some())
}

/// Mark an expired subscription's credits as zeroed (no rollover) without
/// granting a new period. Returns the post-expiry remaining credits (0).
pub async fn expire_subscription(
    client: &Client,
    user_id: &str,
) -> Result<(), tokio_postgres::Error> {
    client
        .execute(
            "UPDATE subscriptions SET status = 'expired', credits_remaining = 0, updated_at = NOW()
             WHERE user_id = $1 AND period_end < NOW()",
            &[&user_id],
        )
        .await?;
    Ok(())
}

/// Atomically record usage and deduct credits. Returns the new balance.
/// The balance decrement and ledger insert happen in a single statement (CTE)
/// so they cannot drift apart even without an outer transaction.
#[allow(clippy::too_many_arguments)]
pub async fn record_usage_and_deduct(
    client: &Client,
    user_id: &str,
    model: &str,
    prompt_tokens: i32,
    completion_tokens: i32,
    cost_micros: i64,
    credits_charged: i32,
    generation_id: Option<&str>,
) -> Result<i32, tokio_postgres::Error> {
    let usage_id = Uuid::new_v4().to_string();
    client
        .execute(
            "INSERT INTO usage_events
                (id, user_id, model, prompt_tokens, completion_tokens,
                 openrouter_cost_micros, credits_charged, openrouter_generation_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &usage_id,
                &user_id,
                &model,
                &prompt_tokens,
                &completion_tokens,
                &cost_micros,
                &credits_charged,
                &generation_id,
            ],
        )
        .await?;

    let ledger_id = Uuid::new_v4().to_string();
    let row = client
        .query_one(
            "WITH upd AS (
                UPDATE subscriptions
                SET credits_remaining = credits_remaining - $2, updated_at = NOW()
                WHERE user_id = $1
                RETURNING credits_remaining
             )
             INSERT INTO credit_ledger (id, user_id, delta, reason, balance_after, usage_id)
             SELECT $3, $1, -$2, 'consume', credits_remaining, $4 FROM upd
             RETURNING balance_after",
            &[&user_id, &credits_charged, &ledger_id, &usage_id],
        )
        .await?;

    Ok(row.get(0))
}

// ─── Payment orders ───

#[derive(Debug)]
pub struct OrderRow {
    pub id: String,
    pub user_id: String,
    pub plan: String,
    pub status: String,
}

pub async fn create_payment_order(
    client: &Client,
    user_id: &str,
    plan: &str,
    amount_cents: i32,
    provider: &str,
) -> Result<String, tokio_postgres::Error> {
    let id = Uuid::new_v4().to_string();
    client
        .execute(
            "INSERT INTO payment_orders (id, user_id, plan, amount_cents, status, provider)
             VALUES ($1, $2, $3, $4, 'pending', $5)",
            &[&id, &user_id, &plan, &amount_cents, &provider],
        )
        .await?;
    Ok(id)
}

pub async fn get_payment_order(
    client: &Client,
    id: &str,
) -> Result<Option<OrderRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT id, user_id, plan, status FROM payment_orders WHERE id = $1",
            &[&id],
        )
        .await?;
    Ok(row.map(|r| OrderRow {
        id: r.get(0),
        user_id: r.get(1),
        plan: r.get(2),
        status: r.get(3),
    }))
}

/// Mark an order paid and bind the provider's order id. Returns false if the
/// order was already paid (idempotent no-op for duplicate callbacks).
pub async fn mark_order_paid(
    client: &Client,
    order_id: &str,
    provider_order_id: &str,
) -> Result<bool, tokio_postgres::Error> {
    let rows = client
        .execute(
            "UPDATE payment_orders
             SET status = 'paid', provider_order_id = $2
             WHERE id = $1 AND status = 'pending'",
            &[&order_id, &provider_order_id],
        )
        .await?;
    Ok(rows > 0)
}

// ─── Device-authorization (CLI OAuth) ───

/// Find a user by id (for minting tokens from a device/refresh grant).
pub async fn find_user_by_id(
    client: &Client,
    id: &str,
) -> Result<Option<UserRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT id, email, name, password_hash, verified, verification_code FROM users WHERE id = $1",
            &[&id],
        )
        .await?;

    Ok(row.map(|r| UserRow {
        id: r.get(0),
        email: r.get(1),
        name: r.get(2),
        password_hash: r.get(3),
        verified: r.get(4),
        verification_code: r.get(5),
    }))
}

#[derive(Debug)]
pub struct DeviceCodeRow {
    pub user_id: Option<String>,
    pub status: String,
    pub expired: bool,
}

pub async fn create_device_code(
    client: &Client,
    device_code: &str,
    user_code: &str,
    expires_secs: i32,
) -> Result<(), tokio_postgres::Error> {
    client
        .execute(
            "INSERT INTO device_codes (device_code, user_code, status, expires_at)
             VALUES ($1, $2, 'pending', NOW() + ($3::int * INTERVAL '1 second'))",
            &[&device_code, &user_code, &expires_secs],
        )
        .await?;
    Ok(())
}

pub async fn get_device_code(
    client: &Client,
    device_code: &str,
) -> Result<Option<DeviceCodeRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT user_id, status, expires_at < NOW() AS expired
             FROM device_codes WHERE device_code = $1",
            &[&device_code],
        )
        .await?;
    Ok(row.map(|r| DeviceCodeRow {
        user_id: r.get(0),
        status: r.get(1),
        expired: r.get(2),
    }))
}

/// Bind an approved device code to a user. Returns false if the code is
/// unknown, already handled, or expired.
pub async fn approve_device_code(
    client: &Client,
    user_code: &str,
    user_id: &str,
) -> Result<bool, tokio_postgres::Error> {
    let rows = client
        .execute(
            "UPDATE device_codes SET status = 'approved', user_id = $2
             WHERE user_code = $1 AND status = 'pending' AND expires_at > NOW()",
            &[&user_code, &user_id],
        )
        .await?;
    Ok(rows > 0)
}

/// Mark a device code consumed so its grant cannot be replayed.
pub async fn claim_device_code(
    client: &Client,
    device_code: &str,
) -> Result<(), tokio_postgres::Error> {
    client
        .execute(
            "UPDATE device_codes SET status = 'claimed' WHERE device_code = $1",
            &[&device_code],
        )
        .await?;
    Ok(())
}

pub async fn store_refresh_token(
    client: &Client,
    user_id: &str,
    token_hash: &str,
    expires_days: i32,
) -> Result<(), tokio_postgres::Error> {
    let id = Uuid::new_v4().to_string();
    client
        .execute(
            "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
             VALUES ($1, $2, $3, NOW() + ($4::int * INTERVAL '1 day'))",
            &[&id, &user_id, &token_hash, &expires_days],
        )
        .await?;
    Ok(())
}

#[derive(Debug)]
pub struct RefreshRow {
    pub user_id: String,
    pub expired: bool,
    pub revoked: bool,
}

pub async fn get_refresh_token(
    client: &Client,
    token_hash: &str,
) -> Result<Option<RefreshRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT user_id, expires_at < NOW() AS expired, revoked
             FROM refresh_tokens WHERE token_hash = $1",
            &[&token_hash],
        )
        .await?;
    Ok(row.map(|r| RefreshRow {
        user_id: r.get(0),
        expired: r.get(1),
        revoked: r.get(2),
    }))
}
