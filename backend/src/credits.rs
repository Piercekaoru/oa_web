//! Credit lifecycle: period reset (no rollover) and plan grants.
//!
//! Balance deduction itself lives in `db::record_usage_and_deduct` (a single
//! atomic statement); this module handles the surrounding policy.

use tokio_postgres::Client;

use crate::db::{self, SubscriptionRow};
use crate::plans;

/// Return the user's subscription, lazily zeroing credits if the period has
/// ended. Credits do not roll over, so an expired period is worth nothing until
/// a new payment grants a fresh allotment.
pub async fn current(
    client: &Client,
    user_id: &str,
) -> Result<Option<SubscriptionRow>, tokio_postgres::Error> {
    let sub = db::get_subscription(client, user_id).await?;
    match sub {
        Some(s) if s.expired && s.status == "active" => {
            db::expire_subscription(client, user_id).await?;
            db::get_subscription(client, user_id).await
        }
        other => Ok(other),
    }
}

/// Grant (or reset to) a plan's 30-day period and credit allotment.
pub async fn grant_plan(
    client: &Client,
    user_id: &str,
    plan_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Pro -> Max upgrade: a second ≤¥1000 payment that tops up the credit delta
    // and flips the plan to Max, keeping the existing period (no reset).
    if plan_key == "max_upgrade" {
        let pro = plans::find("pro").ok_or("missing pro plan")?;
        let max = plans::find("max").ok_or("missing max plan")?;
        let delta = max.credits - pro.credits;
        if !db::upgrade_to_max(client, user_id, delta).await? {
            // No active subscription at fulfillment (Pro lapsed between checkout
            // and payment — very rare). Grant a fresh Max period worth the delta
            // so the payment still yields value.
            db::grant_period(client, user_id, "max", delta).await?;
        }
        return Ok(());
    }

    let plan = plans::find(plan_key).ok_or_else(|| format!("unknown plan: {}", plan_key))?;
    db::grant_period(client, user_id, plan.key, plan.credits).await?;
    Ok(())
}
