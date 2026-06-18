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
}

const PLANS: &[Plan] = &[
    Plan { key: "free", price_cents: 0,     credits: 0,     max_tokens: 4_096,  dynamic: false },
    Plan { key: "plus", price_cents: 2_000, credits: 2_000, max_tokens: 8_192,  dynamic: false },
    Plan { key: "pro",  price_cents: 10_000, credits: 10_000, max_tokens: 16_384, dynamic: false },
    Plan { key: "max",  price_cents: 20_000, credits: 20_000, max_tokens: 32_768, dynamic: true },
];

/// Model ids (or prefixes) reserved for Dynamic-capable plans (Max).
const DYNAMIC_MODEL_PREFIXES: &[&str] = &["openachieve/dynamic", "openachieve/fusion"];

pub fn find(plan_key: &str) -> Option<&'static Plan> {
    PLANS.iter().find(|p| p.key == plan_key)
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
/// Dynamic/fusion models are restricted to plans with `dynamic = true`.
pub fn model_allowed(plan_key: &str, model: &str) -> bool {
    let is_dynamic_model = DYNAMIC_MODEL_PREFIXES
        .iter()
        .any(|prefix| model.starts_with(prefix));
    if !is_dynamic_model {
        return true;
    }
    find(plan_key).map(|p| p.dynamic).unwrap_or(false)
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

/// CPA models offered through the metered proxy (any paid plan). `id` is the
/// branded id clients use; `upstream_id` is sent to the CPA proxy. Credits are
/// charged per 1,000,000 tokens — TODO: tune the rates to your desired pricing.
pub const CPA_CATALOG: &[CpaModel] = &[
    CpaModel { id: "openachieve/grok-build-0.1", upstream_id: "grok-build-0.1", credits_per_mtok_in: 400, credits_per_mtok_out: 1600 },
    CpaModel { id: "openachieve/grok-composer-2.5-fast", upstream_id: "grok-composer-2.5-fast", credits_per_mtok_in: 200, credits_per_mtok_out: 800 },
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
