use crate::models::order::{Capture, Order, Refund};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event_type: String,
    pub event_version: Option<String>,
    pub create_time: String,
    pub resource_type: String,
    pub resource_version: Option<String>,
    pub summary: Option<String>,
    pub resource: serde_json::Value,
    #[serde(default)]
    pub links: Vec<WebhookLink>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookLink {
    pub href: String,
    pub rel: String,
    pub method: Option<String>,
}

#[derive(Debug)]
pub enum ParsedWebhookEvent {
    OrderApproved(Order),
    OrderCompleted(Order),
    PaymentCaptureCompleted(Capture),
    PaymentCaptureDenied(Capture),
    PaymentCapturePending(Capture),
    PaymentCaptureRefunded(Capture),
    RefundCompleted(Refund),
    Unknown {
        event_type: String,
        resource: serde_json::Value,
    },
}

impl WebhookEvent {
    pub fn parse(&self) -> Result<ParsedWebhookEvent, serde_json::Error> {
        match self.event_type.as_str() {
            "CHECKOUT.ORDER.APPROVED" => Ok(ParsedWebhookEvent::OrderApproved(
                serde_json::from_value(self.resource.clone())?,
            )),
            "CHECKOUT.ORDER.COMPLETED" => Ok(ParsedWebhookEvent::OrderCompleted(
                serde_json::from_value(self.resource.clone())?,
            )),
            "PAYMENT.CAPTURE.COMPLETED" => Ok(ParsedWebhookEvent::PaymentCaptureCompleted(
                serde_json::from_value(self.resource.clone())?,
            )),
            "PAYMENT.CAPTURE.DENIED" => Ok(ParsedWebhookEvent::PaymentCaptureDenied(
                serde_json::from_value(self.resource.clone())?,
            )),
            "PAYMENT.CAPTURE.PENDING" => Ok(ParsedWebhookEvent::PaymentCapturePending(
                serde_json::from_value(self.resource.clone())?,
            )),
            "PAYMENT.CAPTURE.REFUNDED" => Ok(ParsedWebhookEvent::PaymentCaptureRefunded(
                serde_json::from_value(self.resource.clone())?,
            )),
            "PAYMENT.REFUND.COMPLETED" => Ok(ParsedWebhookEvent::RefundCompleted(
                serde_json::from_value(self.resource.clone())?,
            )),
            _ => Ok(ParsedWebhookEvent::Unknown {
                event_type: self.event_type.clone(),
                resource: self.resource.clone(),
            }),
        }
    }

    pub fn order_id(&self) -> Option<&str> {
        self.resource.get("id").and_then(|v| v.as_str())
    }

    pub fn is_payment_success(&self) -> bool {
        matches!(
            self.event_type.as_str(),
            "CHECKOUT.ORDER.COMPLETED" | "PAYMENT.CAPTURE.COMPLETED"
        )
    }
}

#[derive(Debug)]
pub struct WebhookHeaders {
    pub transmission_id: String,
    pub transmission_time: String,
    pub cert_url: String,
    pub auth_algo: String,
    pub transmission_sig: String,
}

impl WebhookHeaders {
    pub fn from_headers<'a, I>(headers: I) -> Option<Self>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut transmission_id = None;
        let mut transmission_time = None;
        let mut cert_url = None;
        let mut auth_algo = None;
        let mut transmission_sig = None;

        for (key, value) in headers {
            match key.to_lowercase().as_str() {
                "paypal-transmission-id" => transmission_id = Some(value.to_string()),
                "paypal-transmission-time" => transmission_time = Some(value.to_string()),
                "paypal-cert-url" => cert_url = Some(value.to_string()),
                "paypal-auth-algo" => auth_algo = Some(value.to_string()),
                "paypal-transmission-sig" => transmission_sig = Some(value.to_string()),
                _ => {}
            }
        }

        Some(Self {
            transmission_id: transmission_id?,
            transmission_time: transmission_time?,
            cert_url: cert_url?,
            auth_algo: auth_algo?,
            transmission_sig: transmission_sig?,
        })
    }
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub(crate) struct VerifyWebhookRequest {
    pub webhook_id: String,
    pub transmission_id: String,
    pub transmission_time: String,
    pub cert_url: String,
    pub auth_algo: String,
    pub transmission_sig: String,
    pub webhook_event: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VerifyWebhookResponse {
    pub verification_status: String,
}

impl VerifyWebhookResponse {
    pub fn is_success(&self) -> bool {
        self.verification_status == "SUCCESS"
    }
}

pub mod event_types {
    pub const ORDER_APPROVED: &str = "CHECKOUT.ORDER.APPROVED";
    pub const ORDER_COMPLETED: &str = "CHECKOUT.ORDER.COMPLETED";
    pub const CAPTURE_COMPLETED: &str = "PAYMENT.CAPTURE.COMPLETED";
    pub const CAPTURE_DENIED: &str = "PAYMENT.CAPTURE.DENIED";
    pub const CAPTURE_PENDING: &str = "PAYMENT.CAPTURE.PENDING";
    pub const CAPTURE_REFUNDED: &str = "PAYMENT.CAPTURE.REFUNDED";
    pub const REFUND_COMPLETED: &str = "PAYMENT.REFUND.COMPLETED";
}
