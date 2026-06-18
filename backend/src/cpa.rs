//! Metered proxy to the CPA reverse proxy (OpenAI-compatible).
//!
//! Mirrors `openrouter.rs` but CPA does not return a dollar cost — only token
//! usage. Credits are charged by token count (see `plans::cpa_credits`). Any
//! cost/dollar fields the proxy might add are stripped before reaching clients.

use serde_json::{json, Value};

pub struct CpaResult {
    /// Response body with any cost/usage-dollar fields removed.
    pub body: Value,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
}

fn endpoint(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

fn clamp_max_tokens(body: &mut Value, max_tokens_cap: i32) {
    let requested = body.get("max_tokens").and_then(|v| v.as_i64());
    let capped = match requested {
        Some(n) if n > 0 => n.min(max_tokens_cap as i64),
        _ => max_tokens_cap as i64,
    };
    body["max_tokens"] = json!(capped);
}

/// Strip dollar-cost fields so they never reach the client.
fn sanitize(payload: &mut Value) {
    if let Some(usage) = payload.get_mut("usage").and_then(|u| u.as_object_mut()) {
        usage.remove("cost");
        usage.remove("cost_details");
    }
}

/// Forward a non-streaming request to the CPA proxy and return the sanitized
/// response plus token counts (no dollar cost).
pub async fn forward(
    api_key: &str,
    base_url: &str,
    upstream_id: &str,
    max_tokens_cap: i32,
    mut body: Value,
) -> Result<CpaResult, String> {
    body["model"] = json!(upstream_id);
    body["stream"] = json!(false);
    clamp_max_tokens(&mut body, max_tokens_cap);

    let client = reqwest::Client::new();
    let resp = client
        .post(endpoint(base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("cpa request failed: {}", e))?;

    let status = resp.status();
    let mut payload: Value = resp
        .json()
        .await
        .map_err(|e| format!("invalid cpa response: {}", e))?;

    if !status.is_success() {
        return Err(format!("cpa error {}: {}", status, payload));
    }

    let usage = payload.get("usage").cloned().unwrap_or(Value::Null);
    let prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let completion_tokens = usage.get("completion_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    sanitize(&mut payload);

    Ok(CpaResult {
        body: payload,
        prompt_tokens,
        completion_tokens,
    })
}

/// Forward a streaming request to the CPA proxy. Requests usage in the final
/// SSE chunk (`stream_options.include_usage`) so credits can be settled after.
pub async fn forward_stream(
    api_key: &str,
    base_url: &str,
    upstream_id: &str,
    max_tokens_cap: i32,
    mut body: Value,
) -> Result<reqwest::Response, String> {
    body["model"] = json!(upstream_id);
    body["stream"] = json!(true);
    body["stream_options"] = json!({ "include_usage": true });
    clamp_max_tokens(&mut body, max_tokens_cap);

    let client = reqwest::Client::new();
    let resp = client
        .post(endpoint(base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("cpa request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("cpa error {}: {}", status, text));
    }

    Ok(resp)
}
