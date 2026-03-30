use uuid::Uuid;

use vl_config::StripeConfig;
use vl_core::entities::{CheckoutSession, InvoiceSummary};

/// Stripe REST client implemented via `reqwest` — no extra crate dependencies.
/// Uses the Stripe v1 API with form-encoded payloads.
pub struct StripeService {
    http:            reqwest::Client,
    secret_key:      String,
    webhook_secret:  String,
    success_url:     String,
    cancel_url:      String,
}

impl StripeService {
    pub fn new(config: &StripeConfig) -> Self {
        Self {
            http:           reqwest::Client::new(),
            secret_key:     config.secret_key.clone(),
            webhook_secret: config.webhook_secret.clone(),
            success_url:    config.success_url.clone(),
            cancel_url:     config.cancel_url.clone(),
        }
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn base_url() -> &'static str {
        "https://api.stripe.com/v1"
    }

    async fn post_form(
        &self,
        path: &str,
        params: Vec<(&str, String)>,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let url = format!("{}{}", Self::base_url(), path);
        let resp = self.http
            .post(&url)
            .basic_auth(&self.secret_key, None::<&str>)
            .form(&params)
            .send()
            .await?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            let msg = body["error"]["message"]
                .as_str()
                .unwrap_or("Stripe API error")
                .to_string();
            return Err(anyhow::anyhow!("Stripe {}: {}", status, msg));
        }
        Ok(body)
    }

    async fn get_json(&self, path: &str) -> Result<serde_json::Value, anyhow::Error> {
        let url = format!("{}{}", Self::base_url(), path);
        let resp = self.http
            .get(&url)
            .basic_auth(&self.secret_key, None::<&str>)
            .send()
            .await?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            let msg = body["error"]["message"]
                .as_str()
                .unwrap_or("Stripe API error")
                .to_string();
            return Err(anyhow::anyhow!("Stripe {}: {}", status, msg));
        }
        Ok(body)
    }

    // ── Customer ─────────────────────────────────────────────────────────────

    /// Get or create a Stripe Customer for the given tenant.
    /// Returns the Stripe customer ID (cus_xxx).
    pub async fn get_or_create_customer(
        &self,
        tenant_id: Uuid,
        email:     &str,
        name:      &str,
    ) -> Result<String, anyhow::Error> {
        // Search for existing customer by metadata.tenant_id
        let search_url = format!(
            "/customers/search?query=metadata[%27tenant_id%27]:%27{}%27&limit=1",
            tenant_id
        );
        if let Ok(found) = self.get_json(&search_url).await {
            if let Some(id) = found["data"][0]["id"].as_str() {
                return Ok(id.to_string());
            }
        }

        // Create new customer
        let params = vec![
            ("email",                      email.to_string()),
            ("name",                       name.to_string()),
            ("metadata[tenant_id]",        tenant_id.to_string()),
        ];
        let body = self.post_form("/customers", params).await?;
        let cust_id = body["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No customer ID in Stripe response"))?
            .to_string();

        Ok(cust_id)
    }

    // ── Checkout Session ─────────────────────────────────────────────────────

    /// Create a Stripe Checkout Session for subscription upgrade.
    /// The `tenant_id` is stored in session metadata so the webhook can correlate.
    pub async fn create_checkout_session(
        &self,
        stripe_customer_id: &str,
        stripe_price_id:    &str,
        tenant_id:          Uuid,
        billing_cycle:      &str,
    ) -> Result<CheckoutSession, anyhow::Error> {
        let params = vec![
            ("customer",                           stripe_customer_id.to_string()),
            ("mode",                               "subscription".to_string()),
            ("line_items[0][price]",               stripe_price_id.to_string()),
            ("line_items[0][quantity]",            "1".to_string()),
            ("success_url",                        self.success_url.clone()),
            ("cancel_url",                         self.cancel_url.clone()),
            ("metadata[tenant_id]",                tenant_id.to_string()),
            ("metadata[billing_cycle]",            billing_cycle.to_string()),
            ("subscription_data[metadata][tenant_id]", tenant_id.to_string()),
        ];

        let body = self.post_form("/checkout/sessions", params).await?;

        let session_id = body["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No session ID in Stripe response"))?
            .to_string();
        let session_url = body["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No URL in Stripe checkout session"))?
            .to_string();

        Ok(CheckoutSession { session_id, session_url })
    }

    // ── Billing Portal ───────────────────────────────────────────────────────

    /// Create a Stripe Billing Portal session URL.
    /// The tenant admin opens this URL to manage payment method, invoices, or cancel.
    pub async fn create_portal_session(
        &self,
        stripe_customer_id: &str,
        return_url:         &str,
    ) -> Result<String, anyhow::Error> {
        let params = vec![
            ("customer",   stripe_customer_id.to_string()),
            ("return_url", return_url.to_string()),
        ];

        let body = self.post_form("/billing_portal/sessions", params).await?;
        let url = body["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No URL in Stripe portal session"))?
            .to_string();

        Ok(url)
    }

    // ── Invoices ─────────────────────────────────────────────────────────────

    /// List the last N invoices for a Stripe customer.
    pub async fn list_invoices(
        &self,
        stripe_customer_id: &str,
        limit: u32,
    ) -> Result<Vec<InvoiceSummary>, anyhow::Error> {
        let path = format!(
            "/invoices?customer={}&limit={}&expand[]=data.subscription",
            stripe_customer_id, limit
        );
        let body = self.get_json(&path).await?;

        let items = body["data"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let invoices = items.iter().filter_map(|inv| {
            let id          = inv["id"].as_str()?.to_string();
            let number      = inv["number"].as_str().map(|s| s.to_string());
            let amount_paid = inv["amount_paid"].as_i64().unwrap_or(0);
            let currency    = inv["currency"].as_str().unwrap_or("usd").to_string();
            let status      = inv["status"].as_str().unwrap_or("unknown").to_string();
            let period_start = inv["period_start"].as_i64().unwrap_or(0) * 1000;
            let period_end   = inv["period_end"].as_i64().unwrap_or(0) * 1000;
            let invoice_url  = inv["hosted_invoice_url"].as_str().map(|s| s.to_string());
            let pdf_url      = inv["invoice_pdf"].as_str().map(|s| s.to_string());

            Some(InvoiceSummary {
                id, number, amount_paid, currency, status,
                period_start, period_end, invoice_url, pdf_url,
            })
        }).collect();

        Ok(invoices)
    }

    // ── Webhook signature verification ───────────────────────────────────────

    /// Verify the `Stripe-Signature` header HMAC-SHA256 and return the parsed JSON body.
    pub fn verify_webhook(
        &self,
        payload: &[u8],
        signature_header: &str,
    ) -> Result<serde_json::Value, anyhow::Error> {
        use sha2::{Sha256, Digest};

        // Parse "t=timestamp,v1=hex_signature" header
        let mut timestamp_str = "";
        let mut sig_hex       = "";
        for part in signature_header.split(',') {
            if let Some(v) = part.strip_prefix("t=")  { timestamp_str = v; }
            if let Some(v) = part.strip_prefix("v1=") { sig_hex = v; }
        }
        if timestamp_str.is_empty() || sig_hex.is_empty() {
            return Err(anyhow::anyhow!("Invalid Stripe-Signature header"));
        }

        // Replay protection: reject events older than 5 minutes
        let ts: i64 = timestamp_str.parse().unwrap_or(0);
        if (chrono::Utc::now().timestamp() - ts).abs() > 300 {
            return Err(anyhow::anyhow!("Stripe webhook timestamp too old"));
        }

        // Compute HMAC-SHA256 manually: SHA256(key XOR opad || SHA256(key XOR ipad || message))
        // Stripe signed payload = "timestamp.raw_body"
        let key = self.webhook_secret.as_bytes();
        let signed_payload = format!("{}.{}", timestamp_str, String::from_utf8_lossy(payload));

        // Pad key to 64-byte block size
        let mut k = [0u8; 64];
        if key.len() <= 64 {
            k[..key.len()].copy_from_slice(key);
        } else {
            let h = Sha256::digest(key);
            k[..32].copy_from_slice(&h);
        }

        let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
        let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();

        let mut inner = Sha256::new();
        inner.update(&ipad);
        inner.update(signed_payload.as_bytes());
        let inner_hash = inner.finalize();

        let mut outer = Sha256::new();
        outer.update(&opad);
        outer.update(&inner_hash);
        let mac_bytes = outer.finalize();

        // Encode expected HMAC to lowercase hex
        let expected_hex: String = mac_bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        if expected_hex != sig_hex {
            return Err(anyhow::anyhow!("Stripe webhook signature mismatch"));
        }

        let event: serde_json::Value = serde_json::from_slice(payload)?;
        Ok(event)
    }
}
