//! Subscription plan configuration and credit math.
//!
//! Pricing is internally consistent at 100 credits = $1 of subscription
//! revenue. The only economic lever is the markup multiplier applied when
//! converting real OpenRouter cost into credits charged.
//!
//! Markup M = 2.5  =>  1 credit ≈ $0.004 of real cost  =>  $1 real cost = 250 credits.

/// Reject new proxy requests once the balance drops to/below this many credits,
/// leaving headroom so a single in-flight request cannot drive the balance far
/// negative (deduction is settled only after generation completes).
pub const LOW_BALANCE_BUFFER: i32 = 100;

/// 1 credit = $0.004 of real OpenRouter cost = 4000 micro-USD.
const MICROS_PER_CREDIT: i64 = 4_000;

pub struct Plan {
    pub key: &'static str,
    pub price_cents: i32,
    pub credits: i32,
    pub max_tokens: i32,
    /// Whether this plan may use Dynamic fusion / premium-gated models.
    pub dynamic: bool,
    /// Entry tier restricted to CPA-proxy models only (no OpenRouter/dynamic).
    pub cpa_only: bool,
}

const PLANS: &[Plan] = &[
    Plan { key: "free", price_cents: 0,     credits: 0,     max_tokens: 4_096,  dynamic: false, cpa_only: false },
    Plan { key: "go",   price_cents: 300,   credits: 1_000,  max_tokens: 8_192,  dynamic: false, cpa_only: true },
    Plan { key: "plus", price_cents: 2_000, credits: 2_000, max_tokens: 8_192,  dynamic: false, cpa_only: false },
    Plan { key: "pro",  price_cents: 10_000, credits: 10_000, max_tokens: 16_384, dynamic: false, cpa_only: false },
    Plan { key: "max",  price_cents: 20_000, credits: 20_000, max_tokens: 32_768, dynamic: true,  cpa_only: false },
];

/// Model ids (or prefixes) reserved for Dynamic-capable plans (Max).
const DYNAMIC_MODEL_PREFIXES: &[&str] = &["openachieve/dynamic", "openachieve/fusion"];

pub fn find(plan_key: &str) -> Option<&'static Plan> {
    PLANS.iter().find(|p| p.key == plan_key)
}

/// Tier ordering for upgrade/downgrade checks (higher = more). Unknown/free = 0.
pub fn rank(plan_key: &str) -> i32 {
    match plan_key {
        "go" => 1,
        "plus" => 2,
        "pro" => 3,
        "max" => 4,
        _ => 0,
    }
}

/// Convert real OpenRouter cost (micro-USD) into credits to charge.
/// Rounds to the nearest credit; a non-zero cost always charges at least 1.
pub fn cost_to_credits(cost_micros: i64) -> i32 {
    if cost_micros <= 0 {
        return 0;
    }
    let credits = (cost_micros + MICROS_PER_CREDIT / 2) / MICROS_PER_CREDIT;
    credits.max(1) as i32
}

/// Whether the given model is permitted for the given plan.
/// `cpa_only` plans (Go) may use CPA models exclusively; Dynamic/fusion models
/// are restricted to plans with `dynamic = true`; everything else is open.
pub fn model_allowed(plan_key: &str, model: &str) -> bool {
    let plan = match find(plan_key) {
        Some(p) => p,
        None => return false,
    };
    if plan.cpa_only {
        return find_cpa(model).is_some();
    }
    let is_dynamic_model = DYNAMIC_MODEL_PREFIXES
        .iter()
        .any(|prefix| model.starts_with(prefix));
    if is_dynamic_model {
        return plan.dynamic;
    }
    true
}

pub struct CatalogModel {
    pub id: &'static str,
}

/// Models offered through the metered proxy, surfaced at `GET /api/v1/models`.
/// Ids are forwarded to OpenRouter verbatim — edit this list to match the
/// OpenRouter catalog you want to resell.
pub const CATALOG: &[CatalogModel] = &[
    // OpenRouter models (billed by real cost). Add real OpenRouter ids here.
    // CatalogModel { id: "openai/gpt-4o" },
    // CatalogModel { id: "deepseek/deepseek-chat" },
];

/// A model served via the CPA reverse proxy (OpenAI-compatible) instead of
/// OpenRouter. CPA does not return a dollar cost, so credits are charged by
/// token count using the per-model rates below (credits per 1,000,000 tokens).
pub struct CpaModel {
    /// Public/branded id clients select (e.g. "openachieve/grok-build-0.1").
    pub id: &'static str,
    /// Real id sent to the CPA proxy (e.g. "grok-build-0.1").
    pub upstream_id: &'static str,
    pub credits_per_mtok_in: i64,
    pub credits_per_mtok_out: i64,
}

/// CPA models offered through the metered proxy (available to any paid plan,
/// including Go). To add a CPA model, append one row below and redeploy
/// (`cd backend && cargo check && cargo test`, then `bash 更新.sh`):
///
///   CpaModel {
///       id: "openachieve/<brand>",   // branded id clients select; must start with "openachieve/"
///       upstream_id: "<cpa-real-id>", // the model name as it exists on the CPA proxy
///       credits_per_mtok_in:  <credits charged per 1,000,000 input tokens>,
///       credits_per_mtok_out: <credits charged per 1,000,000 output tokens>,
///   },
///
/// Pricing guide: 100 credits = $1 of subscription revenue; 1 credit ≈ $0.004
/// real cost (markup 2.5). Deductions ceil and charge ≥1 on any non-zero usage.
/// A new entry shows up automatically at `GET /api/v1/models` and routes through
/// the CPA proxy in `chat_completions`.
pub const CPA_CATALOG: &[CpaModel] = &[
    CpaModel { id: "openachieve/grok-build-0.1", upstream_id: "grok-build-0.1", credits_per_mtok_in: 400, credits_per_mtok_out: 1600 },
    CpaModel { id: "openachieve/grok-composer-2.5-fast", upstream_id: "grok-composer-2.5-fast", credits_per_mtok_in: 200, credits_per_mtok_out: 800 },
    // GPT-4.1: priced the same as grok-composer-2.5-fast.
    CpaModel { id: "openachieve/gpt-4.1", upstream_id: "gpt-4.1", credits_per_mtok_in: 200, credits_per_mtok_out: 800 },
];

/// Find a CPA model by id. A hit routes the request to the CPA upstream.
pub fn find_cpa(model: &str) -> Option<&'static CpaModel> {
    CPA_CATALOG.iter().find(|m| m.id == model)
}

/// Credits for a CPA generation: ceil((in_tokens*in_rate + out_tokens*out_rate)
/// / 1_000_000). Any non-zero usage charges at least 1 credit.
pub fn cpa_credits_from_rates(
    in_per_mtok: i64,
    out_per_mtok: i64,
    prompt_tokens: i32,
    completion_tokens: i32,
) -> i32 {
    let total = prompt_tokens as i64 * in_per_mtok + completion_tokens as i64 * out_per_mtok;
    if total <= 0 {
        return 0;
    }
    let credits = (total + 999_999) / 1_000_000;
    credits.max(1) as i32
}

pub fn cpa_credits(m: &CpaModel, prompt_tokens: i32, completion_tokens: i32) -> i32 {
    cpa_credits_from_rates(m.credits_per_mtok_in, m.credits_per_mtok_out, prompt_tokens, completion_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_orders_tiers() {
        assert_eq!(rank("free"), 0);
        assert_eq!(rank("go"), 1);
        assert_eq!(rank("plus"), 2);
        assert_eq!(rank("pro"), 3);
        assert_eq!(rank("max"), 4);
        assert_eq!(rank("anything-unknown"), 0);
        assert!(rank("free") < rank("go"));
        assert!(rank("go") < rank("plus"));
        assert!(rank("plus") < rank("pro"));
        assert!(rank("pro") < rank("max"));
    }

    #[test]
    fn find_returns_known_plans_only() {
        assert!(find("nope").is_none());
        assert!(find("pro").is_some());
        assert!(!find("pro").unwrap().dynamic);
        assert!(find("max").unwrap().dynamic, "only Max may use dynamic models");
    }

    #[test]
    fn go_is_a_three_dollar_cpa_only_tier() {
        let go = find("go").expect("go plan exists");
        assert_eq!(go.price_cents, 300);
        assert_eq!(go.credits, 1_000);
        assert!(go.cpa_only, "Go is restricted to CPA models");
        assert!(!go.dynamic, "Go cannot use dynamic models");
    }

    #[test]
    fn model_allowed_gates_go_to_cpa_only() {
        // Go may only use CPA-catalog models.
        assert!(model_allowed("go", "openachieve/grok-composer-2.5-fast"));
        assert!(!model_allowed("go", "openai/gpt-4o"));
        assert!(!model_allowed("go", "openachieve/dynamic-pro"));
    }

    #[test]
    fn pro_to_max_upgrade_delta_is_100_usd_and_10k_credits() {
        let pro = find("pro").unwrap();
        let max = find("max").unwrap();
        // The upgrade order charges, and grants, exactly the Max−Pro delta.
        assert_eq!(max.price_cents - pro.price_cents, 10_000); // $100.00
        assert_eq!(max.credits - pro.credits, 10_000);
    }

    #[test]
    fn cost_to_credits_rounds_to_nearest_with_min_one() {
        assert_eq!(cost_to_credits(0), 0);
        assert_eq!(cost_to_credits(-5), 0);
        assert_eq!(cost_to_credits(1), 1); // any non-zero cost charges at least 1
        assert_eq!(cost_to_credits(2_000), 1); // 0.5 credit rounds up
        assert_eq!(cost_to_credits(4_000), 1); // exactly 1 credit ($0.004)
        assert_eq!(cost_to_credits(6_000), 2); // 1.5 rounds up
        assert_eq!(cost_to_credits(10_000), 3); // 2.5 rounds up
    }

    #[test]
    fn cpa_credits_ceils_with_min_one() {
        assert_eq!(cpa_credits_from_rates(1, 0, 0, 0), 0);
        assert_eq!(cpa_credits_from_rates(1, 0, 1, 0), 1); // tiny usage -> 1
        assert_eq!(cpa_credits_from_rates(1, 0, 1_000_000, 0), 1); // exactly 1M
        assert_eq!(cpa_credits_from_rates(1, 0, 1_000_001, 0), 2); // ceils
        // Mixed in/out rates.
        assert_eq!(cpa_credits_from_rates(400, 1_600, 1_000_000, 0), 400);
    }

    #[test]
    fn cpa_credits_uses_model_rates() {
        let m = find_cpa("openachieve/grok-build-0.1").expect("catalog model exists");
        assert_eq!(cpa_credits(m, 0, 0), 0);
        // 1M prompt tokens at 400 credits/Mtok in = 400 credits.
        assert_eq!(cpa_credits(m, 1_000_000, 0), 400);
    }

    #[test]
    fn model_allowed_gates_dynamic_to_max() {
        // Non-dynamic models are open to every plan.
        assert!(model_allowed("free", "openai/gpt-4o"));
        assert!(model_allowed("plus", "openachieve/grok-build-0.1"));
        // Dynamic / fusion models are Max-only.
        assert!(model_allowed("max", "openachieve/dynamic-pro"));
        assert!(model_allowed("max", "openachieve/fusion-x"));
        assert!(!model_allowed("pro", "openachieve/dynamic-pro"));
        assert!(!model_allowed("free", "openachieve/fusion-x"));
        // Unknown plan never gets dynamic access.
        assert!(!model_allowed("bogus", "openachieve/dynamic-pro"));
    }
}
