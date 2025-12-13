//! PayPal REST API client for Rust

pub mod api;
pub mod client;
pub mod error;
pub mod models;

pub use client::{Environment, PayPal};
pub use error::{Error, Result};
pub use models::common::Currency;
pub use models::order::{Capture, Item, ItemCategory, Order, OrderStatus, Refund};
pub use models::webhook::{ParsedWebhookEvent, WebhookEvent, WebhookHeaders};

pub mod prelude {
    pub use crate::client::{Environment, PayPal};
    pub use crate::error::{Error, Result};
    pub use crate::models::common::{
        Currency, Intent, LandingPage, Money, ShippingPreference, UserAction,
    };
    pub use crate::models::order::{
        Capture, CaptureStatus, Item, ItemCategory, Order, OrderStatus, Refund, RefundStatus,
    };
    pub use crate::models::webhook::{
        event_types, ParsedWebhookEvent, WebhookEvent, WebhookHeaders,
    };
}
