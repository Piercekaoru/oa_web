use serde::Serialize;

#[derive(Serialize)]
struct ResendEmailRequest<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: String,
}

/// Send a verification email via the Resend API.
pub async fn send_verification_email(
    api_key: &str,
    from: &str,
    to: &str,
    code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let html = format!(
        r#"
        <div style="font-family: 'Inter', -apple-system, sans-serif; max-width: 480px; margin: 0 auto; padding: 40px 20px;">
            <h1 style="color: #fff; font-size: 32px; font-weight: 600; margin-bottom: 8px;">OpenAchieve</h1>
            <p style="color: #888; font-size: 14px; margin-bottom: 32px;">Verify your email address</p>
            <div style="background: #111; border: 1px solid #222; border-radius: 16px; padding: 32px; text-align: center;">
                <p style="color: #aaa; font-size: 14px; margin-bottom: 16px;">Your verification code is:</p>
                <div style="font-size: 48px; font-weight: 700; letter-spacing: 8px; color: #4B66D1; font-family: monospace;">{}</div>
                <p style="color: #666; font-size: 12px; margin-top: 24px;">This code expires in 30 minutes.</p>
            </div>
            <p style="color: #555; font-size: 12px; margin-top: 24px; text-align: center;">
                If you didn't request this, you can safely ignore this email.
            </p>
        </div>
        "#,
        code
    );

    let payload = ResendEmailRequest {
        from,
        to: vec![to],
        subject: "Your OpenAchieve Verification Code",
        html,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("📧 Verification email sent to {}", to);
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("📧 Failed to send email: {} — {}", status, body);
    }

    Ok(())
}
