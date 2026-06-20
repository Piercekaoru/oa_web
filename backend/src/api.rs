//! Account, billing, and metered-proxy HTTP handlers.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::auth_middleware::AuthUser;
use crate::{cpa, credits, db, openrouter, payment, plans, AppState};

/// GET /api/me — current account + subscription (credits only, never dollars).
pub async fn me(state: web::Data<AppState>, user: AuthUser) -> HttpResponse {
    let sub = match credits::current(&state.db, &user.id).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(json!({ "success": false }));
        }
    };

    let (plan, credits_remaining, expires) = match sub {
        Some(s) if s.status == "active" => (s.plan, s.credits_remaining, Some(s.period_end)),
        _ => ("free".to_string(), 0, None),
    };

    HttpResponse::Ok().json(json!({
        "success": true,
        "user": { "id": user.id, "email": user.email, "name": user.name },
        "subscription": { "plan": plan, "credits": credits_remaining, "expires": expires }
    }))
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub plan: String,
    /// Optional FovPay payment method (alipay/wxpay/usdt/paypal); defaults to alipay.
    #[serde(default)]
    pub paytype: Option<String>,
}

/// POST /api/checkout — create a payment order for a paid plan.
pub async fn checkout(
    state: web::Data<AppState>,
    user: AuthUser,
    req: HttpRequest,
    body: web::Json<CheckoutRequest>,
) -> HttpResponse {
    // Resolve the order's plan key, USD price, and subject. "max_upgrade" is a
    // synthetic order type: an active Pro pays the Max−Pro delta ($100 → ¥720,
    // under FovPay's ¥1000 channel limit) to upgrade in place.
    let (order_plan, price_cents, subject): (&str, i32, String) = if body.plan == "max_upgrade" {
        let sub = match credits::current(&state.db, &user.id).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("DB error: {}", e);
                return HttpResponse::InternalServerError().json(json!({ "success": false }));
            }
        };
        if !matches!(&sub, Some(s) if s.status == "active" && s.plan == "pro") {
            return HttpResponse::BadRequest()
                .json(json!({ "success": false, "message": "请先订阅 Pro 再升级 Max。" }));
        }
        match (plans::find("pro"), plans::find("max")) {
            (Some(pro), Some(max)) => (
                "max_upgrade",
                max.price_cents - pro.price_cents,
                "OpenAchieve Max upgrade".to_string(),
            ),
            _ => return HttpResponse::InternalServerError().json(json!({ "success": false })),
        }
    } else {
        let plan = match plans::find(&body.plan) {
            Some(p) if p.price_cents > 0 => p,
            _ => {
                return HttpResponse::BadRequest()
                    .json(json!({ "success": false, "message": "Invalid plan." }))
            }
        };
        // Reject same-or-lower tier: grant_period overwrites credits and resets the
        // period, so a downgrade or repurchase would destroy the current balance.
        let cur_rank = match credits::current(&state.db, &user.id).await {
            Ok(Some(s)) if s.status == "active" => plans::rank(&s.plan),
            Ok(_) => 0,
            Err(e) => {
                eprintln!("DB error: {}", e);
                return HttpResponse::InternalServerError().json(json!({ "success": false }));
            }
        };
        if plans::rank(plan.key) <= cur_rank {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "message": "You're already on this plan or a higher one."
            }));
        }
        (plan.key, plan.price_cents, format!("OpenAchieve {} plan", plan.key))
    };

    let order_id = match db::create_payment_order(
        &state.db,
        &user.id,
        order_plan,
        price_cents,
        payment::PROVIDER,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(json!({ "success": false }));
        }
    };

    // Plans are priced in USD; FovPay settles in CNY. Convert at the configured rate.
    let total_amount = format!("{:.2}", (price_cents as f64 / 100.0) * state.usd_cny_rate);
    let paytype = body.paytype.clone().unwrap_or_else(|| "alipay".to_string());
    let notify_url = format!("{}/api/webhooks/payment", state.public_base_url.trim_end_matches('/'));
    let return_url = format!("{}/#dashboard", state.frontend_base_url.trim_end_matches('/'));
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("0.0.0.0")
        .to_string();

    match payment::create_order(
        &state.fovpay_pid,
        &state.fovpay_key,
        &order_id,
        &total_amount,
        &subject,
        &paytype,
        &notify_url,
        &return_url,
        &client_ip,
    )
    .await
    {
        Ok(pay_url) => HttpResponse::Ok().json(json!({
            "success": true,
            "order_id": order_id,
            "pay_url": pay_url
        })),
        Err(e) => {
            eprintln!("FovPay error: {}", e);
            HttpResponse::BadGateway()
                .json(json!({ "success": false, "message": "Payment provider error." }))
        }
    }
}

/// GET /api/orders/{id} — poll order status (callbacks are async).
pub async fn order_status(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<String>,
) -> HttpResponse {
    let order_id = path.into_inner();
    match db::get_payment_order(&state.db, &order_id).await {
        Ok(Some(o)) if o.user_id == user.id => HttpResponse::Ok()
            .json(json!({ "success": true, "status": o.status, "plan": o.plan })),
        Ok(_) => HttpResponse::NotFound().json(json!({ "success": false })),
        Err(e) => {
            eprintln!("DB error: {}", e);
            HttpResponse::InternalServerError().json(json!({ "success": false }))
        }
    }
}

/// POST /api/webhooks/payment — FovPay async callback (form-urlencoded).
/// Verifies the MD5 signature, grants credits on TRADE_SUCCESS (idempotent),
/// and acknowledges with the literal "success" so FovPay stops retrying.
pub async fn payment_webhook(
    state: web::Data<AppState>,
    form: web::Form<HashMap<String, String>>,
) -> HttpResponse {
    let params = form.into_inner();
    let cb = match payment::verify_callback(&params, &state.fovpay_key) {
        Ok(cb) => cb,
        Err(e) => return HttpResponse::BadRequest().body(e),
    };

    // Verified but not a completed payment (e.g. TRADE_FREEZE): ack without granting.
    if cb.trade_status != "TRADE_SUCCESS" {
        return HttpResponse::Ok().body("success");
    }

    let order = match db::get_payment_order(&state.db, &cb.order_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return HttpResponse::NotFound().body("unknown order"),
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().body("db error");
        }
    };

    // Idempotent: only the first callback that flips pending->paid grants credits.
    match db::mark_order_paid(&state.db, &order.id, &cb.provider_order_id).await {
        Ok(true) => {
            if let Err(e) = credits::grant_plan(&state.db, &order.user_id, &order.plan).await {
                eprintln!("grant error: {}", e);
                return HttpResponse::InternalServerError().body("grant failed");
            }
        }
        Ok(false) => {} // already processed
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().body("db error");
        }
    }

    HttpResponse::Ok().body("success")
}

/// GET /api/v1/models — OpenAI-compatible model list, filtered by plan.
pub async fn models(state: web::Data<AppState>, user: AuthUser) -> HttpResponse {
    let plan = match credits::current(&state.db, &user.id).await {
        Ok(Some(s)) if s.status == "active" => s.plan,
        _ => "free".to_string(),
    };

    let mut data: Vec<Value> = plans::CATALOG
        .iter()
        .filter(|m| plans::model_allowed(&plan, m.id))
        .map(|m| json!({ "id": m.id, "object": "model", "owned_by": "openachieve" }))
        .collect();

    // CPA models are available to any paid plan.
    if plan != "free" {
        for m in plans::CPA_CATALOG {
            data.push(json!({ "id": m.id, "object": "model", "owned_by": "openachieve" }));
        }
    }

    HttpResponse::Ok().json(json!({ "object": "list", "data": data }))
}

/// POST /api/v1/chat/completions — metered OpenRouter proxy.
pub async fn chat_completions(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<Value>,
) -> HttpResponse {
    // 1. Require an active paid subscription with credits above the buffer.
    let sub = match credits::current(&state.db, &user.id).await {
        Ok(Some(s)) if s.status == "active" => s,
        Ok(_) => {
            return HttpResponse::PaymentRequired()
                .json(json!({ "error": "No active subscription." }))
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return HttpResponse::InternalServerError().json(json!({ "error": "server error" }));
        }
    };

    if sub.credits_remaining <= plans::LOW_BALANCE_BUFFER {
        return HttpResponse::PaymentRequired()
            .json(json!({ "error": "Insufficient credits.", "credits": sub.credits_remaining }));
    }

    let plan = match plans::find(&sub.plan) {
        Some(p) => p,
        None => {
            return HttpResponse::InternalServerError().json(json!({ "error": "bad plan" }))
        }
    };

    // 2. Enforce model gating (Dynamic/fusion models are Max-only).
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("");
    if !plans::model_allowed(&sub.plan, model) {
        return HttpResponse::Forbidden()
            .json(json!({ "error": "This model is not available on your plan." }));
    }

    // 3. Pick the upstream: CPA models go to the CPA proxy (charged by token),
    //    everything else to OpenRouter (charged by real cost).
    let cpa_model = plans::find_cpa(model);
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);

    if is_stream {
        let (resp, pricing, display_model) = if let Some(c) = cpa_model {
            (
                cpa::forward_stream(&state.cpa_api_key, &state.cpa_base_url, c.upstream_id, plan.max_tokens, body.into_inner()).await,
                Pricing::CpaTokens,
                Some(c.id.to_string()),
            )
        } else {
            (
                openrouter::forward_stream(&state.openrouter_api_key, plan.max_tokens, body.into_inner()).await,
                Pricing::OpenRouterCost,
                None,
            )
        };
        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                eprintln!("upstream error: {}", e);
                return HttpResponse::BadGateway().json(json!({ "error": "upstream error" }));
            }
        };
        return stream_completion(state.clone(), user.id.clone(), resp, pricing, display_model).await;
    }

    // 4. Non-streaming, CPA upstream (charged by token count).
    if let Some(c) = cpa_model {
        let mut result = match cpa::forward(
            &state.cpa_api_key,
            &state.cpa_base_url,
            c.upstream_id,
            plan.max_tokens,
            body.into_inner(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("CPA error: {}", e);
                return HttpResponse::BadGateway().json(json!({ "error": "upstream error" }));
            }
        };
        let charged = plans::cpa_credits(result.prompt_tokens, result.completion_tokens);
        // Present the branded id to the client, not the upstream model.
        if let Some(obj) = result.body.as_object_mut() {
            obj.insert("model".to_string(), json!(c.id));
        }
        if let Err(e) = db::record_usage_and_deduct(
            &state.db,
            &user.id,
            c.id,
            result.prompt_tokens,
            result.completion_tokens,
            0,
            charged,
            None,
        )
        .await
        {
            eprintln!("deduct error: {}", e);
        }
        return HttpResponse::Ok().json(result.body);
    }

    // 4b. Non-streaming, OpenRouter upstream (charged by real cost).
    let result = match openrouter::forward(
        &state.openrouter_api_key,
        plan.max_tokens,
        body.into_inner(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("OpenRouter error: {}", e);
            return HttpResponse::BadGateway().json(json!({ "error": "upstream error" }));
        }
    };

    // 5. Settle: convert real cost -> credits and deduct atomically.
    let charged = plans::cost_to_credits(result.cost_micros);
    if let Err(e) = db::record_usage_and_deduct(
        &state.db,
        &user.id,
        &result.model,
        result.prompt_tokens,
        result.completion_tokens,
        result.cost_micros,
        charged,
        result.generation_id.as_deref(),
    )
    .await
    {
        eprintln!("deduct error: {}", e);
    }

    // 6. Return sanitized output (no dollar fields).
    HttpResponse::Ok().json(result.body)
}

/// Per-upstream credit pricing for the streaming settle step.
enum Pricing {
    OpenRouterCost,
    CpaTokens,
}

/// Relay an upstream SSE stream to the client, stripping dollar fields from each
/// chunk and settling credits once the stream finishes.
async fn stream_completion(
    state: web::Data<AppState>,
    user_id: String,
    resp: reqwest::Response,
    pricing: Pricing,
    display_model: Option<String>,
) -> HttpResponse {
    let sse = async_stream::stream! {
        use futures_util::StreamExt;

        let mut upstream = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut cost_micros: i64 = 0;
        let mut prompt_tokens: i32 = 0;
        let mut completion_tokens: i32 = 0;
        let mut model = String::from("unknown");
        let mut gen_id: Option<String> = None;

        while let Some(chunk) = upstream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(_) => {
                    yield Err(std::io::Error::new(std::io::ErrorKind::Other, "upstream stream error"));
                    break;
                }
            };
            buf.extend_from_slice(&chunk);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let mut rewritten: Option<web::Bytes> = None;

                if let Ok(s) = std::str::from_utf8(&line) {
                    let trimmed = s.trim_end_matches(|c| c == '\n' || c == '\r');
                    if let Some(payload) = trimmed.strip_prefix("data: ") {
                        if payload != "[DONE]" {
                            if let Ok(mut v) = serde_json::from_str::<Value>(payload) {
                                if let Some(usage) = v.get("usage") {
                                    if !usage.is_null() {
                                        if let Some(c) = usage.get("cost").and_then(|x| x.as_f64()) {
                                            cost_micros = (c * 1_000_000.0).round() as i64;
                                        }
                                        if let Some(pt) = usage.get("prompt_tokens").and_then(|x| x.as_i64()) {
                                            prompt_tokens = pt as i32;
                                        }
                                        if let Some(ct) = usage.get("completion_tokens").and_then(|x| x.as_i64()) {
                                            completion_tokens = ct as i32;
                                        }
                                    }
                                }
                                if let Some(m) = v.get("model").and_then(|x| x.as_str()) {
                                    model = m.to_string();
                                }
                                if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                                    gen_id = Some(id.to_string());
                                }
                                let mut changed = false;
                                if let Some(dm) = &display_model {
                                    model = dm.clone();
                                    if v.get("model").is_some() {
                                        v["model"] = Value::String(dm.clone());
                                        changed = true;
                                    }
                                }
                                if let Some(uo) = v.get_mut("usage").and_then(|u| u.as_object_mut()) {
                                    changed |= uo.remove("cost").is_some();
                                    changed |= uo.remove("cost_details").is_some();
                                }
                                if changed {
                                    rewritten = Some(web::Bytes::from(format!("data: {}\n", v)));
                                }
                            }
                        }
                    }
                }

                match rewritten {
                    Some(b) => yield Ok(b),
                    None => yield Ok(web::Bytes::from(line)),
                }
            }
        }

        if !buf.is_empty() {
            yield Ok(web::Bytes::from(buf));
        }

        let (charged, cost_micros_final) = match pricing {
            Pricing::OpenRouterCost => (plans::cost_to_credits(cost_micros), cost_micros),
            Pricing::CpaTokens => (plans::cpa_credits(prompt_tokens, completion_tokens), 0),
        };
        if let Err(e) = db::record_usage_and_deduct(
            &state.db,
            &user_id,
            &model,
            prompt_tokens,
            completion_tokens,
            cost_micros_final,
            charged,
            gen_id.as_deref(),
        )
        .await
        {
            eprintln!("deduct error: {}", e);
        }
    };

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(Box::pin(sse))
}
