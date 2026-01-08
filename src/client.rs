use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::OrderBuilder;
use crate::error::{ApiErrorResponse, Error, Result};
use crate::models::{
    CaptureOrderRequest, CreateOrderRequest, Order, Refund, RefundRequest, VerifyWebhookResponse,
    WebhookEvent, WebhookHeaders,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Sandbox,
    Live,
}

impl Environment {
    fn base_url(&self) -> &'static str {
        match self {
            Environment::Sandbox => "https://api-m.sandbox.paypal.com",
            Environment::Live => "https://api-m.paypal.com",
        }
    }
}

#[derive(Debug)]
struct AccessToken {
    token: String,
    expires_at: std::time::Instant,
}

impl AccessToken {
    fn is_expired(&self) -> bool {
        self.expires_at
            .saturating_duration_since(std::time::Instant::now())
            < std::time::Duration::from_secs(60)
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct ClientTokenResponse {
    pub client_token: String,
    #[allow(dead_code)]
    pub expires_in: usize,
}

#[derive(Clone)]
pub struct PayPal {
    inner: Arc<PayPalInner>,
}

struct PayPalInner {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    environment: Environment,
    token: RwLock<Option<AccessToken>>,
    webhook_id: RwLock<Option<String>>,
}

impl PayPal {
    pub async fn sandbox(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self> {
        Self::new(client_id, client_secret, Environment::Sandbox).await
    }

    pub async fn live(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self> {
        Self::new(client_id, client_secret, Environment::Live).await
    }

    pub async fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        environment: Environment,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let paypal = Self {
            inner: Arc::new(PayPalInner {
                client,
                client_id: client_id.into(),
                client_secret: client_secret.into(),
                environment,
                token: RwLock::new(None),
                webhook_id: RwLock::new(None),
            }),
        };

        paypal.authenticate().await?;
        Ok(paypal)
    }

    pub async fn set_webhook_id(&self, webhook_id: impl Into<String>) {
        let mut id = self.inner.webhook_id.write().await;
        *id = Some(webhook_id.into());
    }

    pub fn environment(&self) -> Environment {
        self.inner.environment
    }

    pub fn is_sandbox(&self) -> bool {
        self.inner.environment == Environment::Sandbox
    }

    async fn authenticate(&self) -> Result<()> {
        let url = format!("{}/v1/oauth2/token", self.inner.environment.base_url());

        let credentials = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.inner.client_id, self.inner.client_secret),
        );

        let response = self
            .inner
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Basic {}", credentials))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ApiErrorResponse = response.json().await?;
            return Err(error.into());
        }

        let token_response: TokenResponse = response.json().await?;

        let access_token = AccessToken {
            token: token_response.access_token,
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(token_response.expires_in),
        };

        let mut token = self.inner.token.write().await;
        *token = Some(access_token);

        Ok(())
    }

    async fn get_token(&self) -> Result<String> {
        {
            let token = self.inner.token.read().await;
            if let Some(ref t) = *token {
                if !t.is_expired() {
                    return Ok(t.token.clone());
                }
            }
        }

        self.authenticate().await?;

        let token = self.inner.token.read().await;
        token
            .as_ref()
            .map(|t| t.token.clone())
            .ok_or(Error::TokenExpired)
    }

    async fn auth_headers(&self) -> Result<HeaderMap> {
        let token = self.get_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|_| Error::Config("Invalid token format".into()))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    pub fn create_order(&self) -> OrderBuilder {
        OrderBuilder::new(self.clone())
    }

    pub async fn quick_checkout(&self, amount: f64, currency: crate::Currency) -> Result<Order> {
        self.create_order().amount(amount, currency).send().await
    }

    pub(crate) async fn create_order_request(&self, request: CreateOrderRequest) -> Result<Order> {
        let url = format!("{}/v2/checkout/orders", self.inner.environment.base_url());
        let headers = self.auth_headers().await?;

        let response = self
            .inner
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn get_order(&self, order_id: &str) -> Result<Order> {
        let url = format!(
            "{}/v2/checkout/orders/{}",
            self.inner.environment.base_url(),
            order_id
        );
        let headers = self.auth_headers().await?;

        let response = self.inner.client.get(&url).headers(headers).send().await?;

        self.handle_response(response).await
    }

    #[cfg(not(feature = "v6"))]
    pub async fn client_token(&self) -> Result<String> {
        let url = format!(
            "{}/v1/identity/generate-token",
            self.inner.environment.base_url()
        );
        let headers = self.auth_headers().await?;
        let response = self.inner.client.post(&url).headers(headers).send().await?;
        let token: ClientTokenResponse = self.handle_response(response).await?;
        Ok(token.client_token)
    }

    #[cfg(feature = "v6")]
    pub async fn client_token(&self) -> Result<String> {
        let url = format!("{}/v1/oauth2/token", self.inner.environment.base_url());

        let credentials = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.inner.client_id, self.inner.client_secret),
        );

        let response = self
            .inner
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Basic {}", credentials))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials&response_type=client_token")
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ApiErrorResponse = response.json().await?;
            return Err(error.into());
        }

        let token_response: TokenResponse = response.json().await?;

        let access_token = AccessToken {
            token: token_response.access_token,
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(token_response.expires_in),
        };
        Ok(access_token.token)
    }

    pub async fn capture(&self, order_id: &str) -> Result<Order> {
        let url = format!(
            "{}/v2/checkout/orders/{}/capture",
            self.inner.environment.base_url(),
            order_id
        );
        let headers = self.auth_headers().await?;
        let request = CaptureOrderRequest::default();

        let response = self
            .inner
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub fn refund(&self, capture_id: &str) -> RefundBuilder {
        RefundBuilder::new(self.clone(), capture_id.to_string())
    }

    pub(crate) async fn refund_capture(
        &self,
        capture_id: &str,
        request: RefundRequest,
    ) -> Result<Refund> {
        let url = format!(
            "{}/v2/payments/captures/{}/refund",
            self.inner.environment.base_url(),
            capture_id
        );
        let headers = self.auth_headers().await?;

        let response = self
            .inner
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn verify_webhook<'a, I>(&self, body: &str, headers: I) -> Result<WebhookEvent>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let webhook_headers = WebhookHeaders::from_headers(headers)
            .ok_or_else(|| Error::WebhookVerification("Missing required headers".into()))?;

        let webhook_id = self.inner.webhook_id.read().await;
        let webhook_id = webhook_id
            .as_ref()
            .ok_or_else(|| Error::WebhookVerification("Webhook ID not set".into()))?;

        let webhook_event: serde_json::Value = serde_json::from_str(body)?;
        let verify_request = format!(
            r#"{{"webhook_id":"{}",
            "transmission_id": "{}",
            "transmission_time": "{}",
            "cert_url": "{}",
            "auth_algo": "{}",
            "transmission_sig": "{}",
            "webhook_event":{}}}"#,
            webhook_id.clone(),
            webhook_headers.transmission_id,
            webhook_headers.transmission_time,
            webhook_headers.cert_url,
            webhook_headers.auth_algo,
            webhook_headers.transmission_sig,
            body
        );

        let url = format!(
            "{}/v1/notifications/verify-webhook-signature",
            self.inner.environment.base_url()
        );
        let headers = self.auth_headers().await?;

        let response = self
            .inner
            .client
            .post(&url)
            .headers(headers)
            .header("Content_Type", "application/json")
            .body(verify_request)
            .send()
            .await?;

        let verify_response: VerifyWebhookResponse = self.handle_response(response).await?;
        if !verify_response.is_success() {
            return Err(Error::WebhookVerification(
                "Signature verification failed".into(),
            ));
        }

        let event: WebhookEvent = serde_json::from_value(webhook_event)?;
        Ok(event)
    }

    pub fn parse_webhook_unverified(&self, body: &str) -> Result<WebhookEvent> {
        let event: WebhookEvent = serde_json::from_str(body)?;
        Ok(event)
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_response: ApiErrorResponse = response.json().await?;
            Err(error_response.into())
        }
    }
}

pub struct RefundBuilder {
    client: PayPal,
    capture_id: String,
    amount: Option<crate::models::Money>,
    note: Option<String>,
    invoice_id: Option<String>,
}

impl RefundBuilder {
    pub(crate) fn new(client: PayPal, capture_id: String) -> Self {
        Self {
            client,
            capture_id,
            amount: None,
            note: None,
            invoice_id: None,
        }
    }

    pub async fn full(self) -> Result<Refund> {
        let request = RefundRequest {
            amount: None,
            note_to_payer: self.note,
            invoice_id: self.invoice_id,
        };
        self.client.refund_capture(&self.capture_id, request).await
    }

    pub fn partial(mut self, amount: f64, currency: crate::Currency) -> Self {
        self.amount = Some(crate::models::Money::new(amount, currency));
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn invoice_id(mut self, invoice_id: impl Into<String>) -> Self {
        self.invoice_id = Some(invoice_id.into());
        self
    }

    pub async fn send(self) -> Result<Refund> {
        let request = RefundRequest {
            amount: self.amount,
            note_to_payer: self.note,
            invoice_id: self.invoice_id,
        };
        self.client.refund_capture(&self.capture_id, request).await
    }
}

impl std::fmt::Debug for PayPal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayPal")
            .field("environment", &self.inner.environment)
            .finish_non_exhaustive()
    }
}
