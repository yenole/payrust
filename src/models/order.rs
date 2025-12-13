use crate::models::common::{Intent, LandingPage, Link, Money, ShippingPreference, UserAction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Created,
    Approved,
    Saved,
    Voided,
    Completed,
    PayerActionRequired,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderRequest {
    pub intent: Intent,
    pub purchase_units: Vec<PurchaseUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_source: Option<PaymentSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_context: Option<ApplicationContext>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PurchaseUnit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soft_descriptor: Option<String>,
    pub amount: PurchaseAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<Shipping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<PaymentCollection>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PurchaseAmount {
    pub currency_code: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<AmountBreakdown>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AmountBreakdown {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_total: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handling: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_total: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insurance: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_discount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quantity: String,
    pub unit_amount: Money,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ItemCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax: Option<Money>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemCategory {
    DigitalGoods,
    PhysicalGoods,
    Donation,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Shipping {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ShippingName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingName {
    pub full_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_area_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_area_2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    pub country_code: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ApplicationContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landing_page: Option<LandingPage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_preference: Option<ShippingPreference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_action: Option<UserAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paypal: Option<PayPalWallet>,
}

#[derive(Debug, Default, Serialize)]
pub struct PayPalWallet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experience_context: Option<ExperienceContext>,
}

#[derive(Debug, Default, Serialize)]
pub struct ExperienceContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_preference: Option<ShippingPreference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landing_page: Option<LandingPage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_action: Option<UserAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub id: String,
    pub status: OrderStatus,
    #[serde(default)]
    pub intent: Option<Intent>,
    #[serde(default)]
    pub purchase_units: Vec<PurchaseUnit>,
    #[serde(default)]
    pub links: Vec<Link>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl Order {
    pub fn approve_url(&self) -> Option<&str> {
        self.links
            .iter()
            .find(|l| l.rel == "approve" || l.rel == "payer-action")
            .map(|l| l.href.as_str())
    }

    pub fn capture_url(&self) -> Option<&str> {
        self.links
            .iter()
            .find(|l| l.rel == "capture")
            .map(|l| l.href.as_str())
    }

    pub fn self_url(&self) -> Option<&str> {
        self.links
            .iter()
            .find(|l| l.rel == "self")
            .map(|l| l.href.as_str())
    }

    pub fn is_approved(&self) -> bool {
        self.status == OrderStatus::Approved
    }
    pub fn is_completed(&self) -> bool {
        self.status == OrderStatus::Completed
    }

    pub fn total(&self) -> Option<f64> {
        let mut total = 0.0;
        for pu in &self.purchase_units {
            total += pu.amount.value.parse::<f64>().ok()?;
        }
        Some(total)
    }

    pub fn currency(&self) -> Option<&str> {
        let mut iter = self.purchase_units.iter();
        let first = iter.next()?;
        let code = first.amount.currency_code.as_str();

        if iter.all(|pu| pu.amount.currency_code == code) {
            Some(code)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentCollection {
    #[serde(default)]
    pub captures: Vec<Capture>,
    #[serde(default)]
    pub refunds: Vec<Refund>,
    #[serde(default)]
    pub authorizations: Vec<Authorization>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Capture {
    pub id: String,
    pub status: CaptureStatus,
    #[serde(default)]
    pub amount: Option<Money>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CaptureStatus {
    Completed,
    Declined,
    PartiallyRefunded,
    Pending,
    Refunded,
    Failed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Refund {
    pub id: String,
    pub status: RefundStatus,
    #[serde(default)]
    pub amount: Option<Money>,
    pub create_time: Option<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefundStatus {
    Cancelled,
    Failed,
    Pending,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Authorization {
    pub id: String,
    pub status: AuthorizationStatus,
    #[serde(default)]
    pub amount: Option<Money>,
    pub create_time: Option<String>,
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorizationStatus {
    Created,
    Captured,
    Denied,
    Expired,
    PartiallyCaptured,
    Voided,
    Pending,
}

#[derive(Debug, Default, Serialize)]
pub struct CaptureOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_source: Option<PaymentSource>,
}

#[derive(Debug, Serialize)]
pub struct RefundRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_to_payer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Intent;

    fn make_purchase_unit(currency: &str, value: &str) -> PurchaseUnit {
        PurchaseUnit {
            reference_id: None,
            description: None,
            custom_id: None,
            invoice_id: None,
            soft_descriptor: None,
            amount: PurchaseAmount {
                currency_code: currency.to_string(),
                value: value.to_string(),
                breakdown: None,
            },
            items: None,
            shipping: None,
            payments: None,
        }
    }

    fn base_order(units: Vec<PurchaseUnit>) -> Order {
        Order {
            id: "id".into(),
            status: OrderStatus::Created,
            intent: Some(Intent::Capture),
            purchase_units: units,
            links: vec![],
            create_time: None,
            update_time: None,
        }
    }

    #[test]
    fn total_sums_multiple_units() {
        let units = vec![
            make_purchase_unit("USD", "10.00"),
            make_purchase_unit("USD", "5.50"),
        ];
        let order = base_order(units);

        assert_eq!(order.total(), Some(15.5));
    }

    #[test]
    fn currency_conflict_returns_none() {
        let units = vec![
            make_purchase_unit("USD", "10.00"),
            make_purchase_unit("EUR", "5.00"),
        ];
        let order = base_order(units);

        assert_eq!(order.currency(), None);
    }
}
