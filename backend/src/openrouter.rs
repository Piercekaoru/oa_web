//! Metered proxy to OpenRouter.
//!
//! Forwards a chat/completions request using the platform's master key, enables
//! usage accounting to learn the real dollar cost, and returns the model output
//! with every cost/dollar field stripped — users only ever see credits.
//!
//! v1 handles non-streaming responses. Streaming (`stream: true`) is forced off
//! server-side; SSE passthrough with post-hoc cost lookup is a follow-up.

use serde_json::{json, Value};

const ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

pub struct ForwardResult {
    /// Response body with all cost/usage-dollar fields removed.
    pub body: Value,
    pub cost_micros: i64,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub model: String,
    pub generation_id: Option<String>,
}

/// Forward a request to OpenRouter and return the sanitized response + real cost.
pub async fn forward(
    api_key: &str,
    max_tokens_cap: i32,
    mut body: Value,
) -> Result<ForwardResult, String> {
    // Force usage accounting on, disable streaming (v1), and clamp max_tokens.
    body["usage"] = json!({ "include": true });
    body["stream"] = json!(false);
    let requested = body.get("max_tokens").and_then(|v| v.as_i64());
    let capped = match requested {
        Some(n) if n > 0 => n.min(max_tokens_cap as i64),
        _ => max_tokens_cap as i64,
    };
    body["max_tokens"] = json!(capped);

    let client = reqwest::Client::new();
    let resp = client
        .post(ENDPOINT)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("upstream request failed: {}", e))?;

    let status = resp.status();
    let mut payload: Value = resp
        .json()
        .await
        .map_err(|e| format!("invalid upstream response: {}", e))?;

    if !status.is_success() {
        return Err(format!("upstream error {}: {}", status, payload));
    }

    let usage = payload.get("usage").cloned().unwrap_or(Value::Null);
    let cost_usd = usage.get("cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let cost_micros = (cost_usd * 1_000_000.0).round() as i64;
    let prompt_tokens = usage
        .get("prompt_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let completion_tokens = usage
        .get("completion_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let generation_id = payload
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    sanitize(&mut payload);

    Ok(ForwardResult {
        body: payload,
        cost_micros,
        prompt_tokens,
        completion_tokens,
        model,
        generation_id,
    })
}

/// Strip dollar-cost fields so they never reach the client.
fn sanitize(payload: &mut Value) {
    if let Some(usage) = payload.get_mut("usage").and_then(|u| u.as_object_mut()) {
        usage.remove("cost");
        usage.remove("cost_details");
    }
}

/// Forward a streaming request to OpenRouter. Enables usage accounting and
/// clamps max_tokens; the caller relays the SSE body and parses the final usage
/// chunk for cost. Returns the raw upstream response for streaming.
pub async fn forward_stream(
    api_key: &str,
    max_tokens_cap: i32,
    mut body: Value,
) -> Result<reqwest::Response, String> {
    body["usage"] = json!({ "include": true });
    body["stream"] = json!(true);
    let requested = body.get("max_tokens").and_then(|v| v.as_i64());
    let capped = match requested {
        Some(n) if n > 0 => n.min(max_tokens_cap as i64),
        _ => max_tokens_cap as i64,
    };
    body["max_tokens"] = json!(capped);

    let client = reqwest::Client::new();
    let resp = client
        .post(ENDPOINT)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("upstream request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("upstream error {}: {}", status, text));
    }

    Ok(resp)
}
