//! FovPay payment integration (https://www.fovpay.com).
//!
//! Requests are form-urlencoded and MD5-signed; responses are JSON. The async
//! notify callback is form-urlencoded and must be answered with the literal
//! string "success". Only `trade_status == "TRADE_SUCCESS"` grants credits.

use std::collections::{BTreeMap, HashMap};

pub const PROVIDER: &str = "fovpay";

const CREATE_ENDPOINT: &str = "https://www.fovpay.com/openapi/pay/create";

pub struct CallbackData {
    /// Our order id (FovPay `out_trade_no`).
    pub order_id: String,
    /// FovPay's transaction id (`trade_no`).
    pub provider_order_id: String,
    pub trade_status: String,
}

/// MD5 sign per FovPay rules: drop `sign`/`sign_type` and empty values, sort by
/// key (ASCII), join as `k=v&...`, append `&key=SECRET`, MD5, uppercase hex.
pub fn sign(params: &BTreeMap<String, String>, key: &str) -> String {
    let pairs: Vec<String> = params
        .iter()
        .filter(|(k, v)| k.as_str() != "sign" && k.as_str() != "sign_type" && !v.is_empty())
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    let base = format!("{}&key={}", pairs.join("&"), key);
    format!("{:X}", md5::compute(base))
}

/// Create a payment order at FovPay and return the pay URL the user is sent to.
#[allow(clippy::too_many_arguments)]
pub async fn create_order(
    pid: &str,
    key: &str,
    out_trade_no: &str,
    total_amount_cny: &str,
    subject: &str,
    paytype_code: &str,
    notify_url: &str,
    return_url: &str,
    client_ip: &str,
) -> Result<String, String> {
    let mut params: BTreeMap<String, String> = BTreeMap::new();
    params.insert("pid".into(), pid.to_string());
    params.insert("out_trade_no".into(), out_trade_no.to_string());
    params.insert("total_amount".into(), total_amount_cny.to_string());
    params.insert("subject".into(), subject.to_string());
    params.insert("paytype_code".into(), paytype_code.to_string());
    params.insert("notify_url".into(), notify_url.to_string());
    if !return_url.is_empty() {
        params.insert("return_url".into(), return_url.to_string());
    }
    params.insert("client_ip".into(), client_ip.to_string());
    params.insert("timestamp".into(), chrono::Utc::now().timestamp().to_string());

    let signature = sign(&params, key);
    let mut form: Vec<(String, String)> = params.into_iter().collect();
    form.push(("sign_type".into(), "MD5".into()));
    form.push(("sign".into(), signature));

    let client = reqwest::Client::new();
    let resp = client
        .post(CREATE_ENDPOINT)
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("fovpay request failed: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("fovpay read body failed: {}", e))?;

    let v: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("invalid fovpay response ({}): {} | body: {}", status, e, body))?;

    // FovPay may return numeric fields as numbers or strings; accept both.
    let code = v
        .get("code")
        .and_then(|c| c.as_i64().or_else(|| c.as_str().and_then(|s| s.trim().parse().ok())));
    if code != Some(1) {
        let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or("");
        return Err(format!("fovpay create failed (code={:?}): {} | body: {}", code, msg, body));
    }

    let pay_url = v
        .get("data")
        .and_then(|d| d.get("pay_url"))
        .and_then(|p| p.as_str())
        .unwrap_or("")
        .to_string();
    if pay_url.is_empty() {
        return Err(format!("fovpay returned empty pay_url | body: {}", body));
    }
    Ok(pay_url)
}

/// Verify an async callback's MD5 signature and extract the result.
pub fn verify_callback(params: &HashMap<String, String>, key: &str) -> Result<CallbackData, String> {
    let provided = params.get("sign").cloned().ok_or("missing sign")?;

    let sorted: BTreeMap<String, String> =
        params.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let expected = sign(&sorted, key);
    if expected.to_uppercase() != provided.to_uppercase() {
        return Err("invalid callback signature".into());
    }

    let order_id = params
        .get("out_trade_no")
        .cloned()
        .ok_or("missing out_trade_no")?;
    let provider_order_id = params
        .get("trade_no")
        .cloned()
        .unwrap_or_else(|| order_id.clone());
    let trade_status = params.get("trade_status").cloned().unwrap_or_default();

    Ok(CallbackData {
        order_id,
        provider_order_id,
        trade_status,
    })
}
