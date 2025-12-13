use crate::client::PayPal;
use crate::error::{Error, Result};
use crate::models::{
    AmountBreakdown, ApplicationContext, CreateOrderRequest, Currency, Intent, Item, ItemCategory,
    LandingPage, Money, Order, PurchaseAmount, PurchaseUnit, ShippingPreference, UserAction,
};

pub struct OrderBuilder {
    client: PayPal,
    intent: Intent,
    currency: Currency,
    amount: f64,
    items: Vec<Item>,
    description: Option<String>,
    custom_id: Option<String>,
    invoice_id: Option<String>,
    soft_descriptor: Option<String>,
    return_url: Option<String>,
    cancel_url: Option<String>,
    brand_name: Option<String>,
    locale: Option<String>,
    landing_page: LandingPage,
    shipping_preference: ShippingPreference,
    user_action: UserAction,
    shipping: f64,
    tax: f64,
    discount: f64,
}

impl OrderBuilder {
    pub(crate) fn new(client: PayPal) -> Self {
        Self {
            client,
            intent: Intent::Capture,
            currency: Currency::USD,
            amount: 0.0,
            items: Vec::new(),
            description: None,
            custom_id: None,
            invoice_id: None,
            soft_descriptor: None,
            return_url: None,
            cancel_url: None,
            brand_name: None,
            locale: None,
            landing_page: LandingPage::Login,
            shipping_preference: ShippingPreference::NoShipping,
            user_action: UserAction::PayNow,
            shipping: 0.0,
            tax: 0.0,
            discount: 0.0,
        }
    }

    pub fn amount(mut self, amount: f64, currency: Currency) -> Self {
        self.amount = amount;
        self.currency = currency;
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn item(mut self, name: impl Into<String>, quantity: u32, unit_price: f64) -> Self {
        self.items.push(Item {
            name: name.into(),
            quantity: quantity.to_string(),
            unit_amount: Money::new(unit_price, self.currency),
            description: None,
            sku: None,
            category: None,
            tax: None,
        });
        self
    }

    pub fn item_detailed(
        mut self,
        name: impl Into<String>,
        quantity: u32,
        unit_price: f64,
        description: Option<String>,
        sku: Option<String>,
        category: Option<ItemCategory>,
    ) -> Self {
        self.items.push(Item {
            name: name.into(),
            quantity: quantity.to_string(),
            unit_amount: Money::new(unit_price, self.currency),
            description,
            sku,
            category,
            tax: None,
        });
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn custom_id(mut self, custom_id: impl Into<String>) -> Self {
        self.custom_id = Some(custom_id.into());
        self
    }

    pub fn invoice_id(mut self, invoice_id: impl Into<String>) -> Self {
        self.invoice_id = Some(invoice_id.into());
        self
    }

    pub fn soft_descriptor(mut self, descriptor: impl Into<String>) -> Self {
        self.soft_descriptor = Some(descriptor.into());
        self
    }

    pub fn return_url(mut self, url: impl Into<String>) -> Self {
        self.return_url = Some(url.into());
        self
    }

    pub fn cancel_url(mut self, url: impl Into<String>) -> Self {
        self.cancel_url = Some(url.into());
        self
    }

    pub fn urls(self, return_url: impl Into<String>, cancel_url: impl Into<String>) -> Self {
        self.return_url(return_url).cancel_url(cancel_url)
    }

    pub fn brand_name(mut self, name: impl Into<String>) -> Self {
        self.brand_name = Some(name.into());
        self
    }

    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    pub fn landing_page(mut self, landing_page: LandingPage) -> Self {
        self.landing_page = landing_page;
        self
    }

    pub fn shipping_preference(mut self, preference: ShippingPreference) -> Self {
        self.shipping_preference = preference;
        self
    }

    pub fn user_action(mut self, action: UserAction) -> Self {
        self.user_action = action;
        self
    }

    pub fn shipping(mut self, amount: f64) -> Self {
        self.shipping = amount;
        self
    }

    pub fn tax(mut self, amount: f64) -> Self {
        self.tax = amount;
        self
    }

    pub fn discount(mut self, amount: f64) -> Self {
        self.discount = amount;
        self
    }

    pub fn require_shipping(mut self) -> Self {
        self.shipping_preference = ShippingPreference::GetFromFile;
        self
    }

    pub fn digital_goods(mut self) -> Self {
        self.shipping_preference = ShippingPreference::NoShipping;
        self
    }

    pub async fn send(self) -> Result<Order> {
        let client = self.client.clone();
        let request = self.build_request()?;
        client.create_order_request(request).await
    }

    fn build_request(self) -> Result<CreateOrderRequest> {
        let decimals = self.currency.decimal_places() as u32;
        let item_total_units = self.compute_item_total_units(decimals)?;
        let shipping_units = Self::to_minor_units(self.shipping, decimals);
        let tax_units = Self::to_minor_units(self.tax, decimals);
        let discount_units = Self::to_minor_units(self.discount, decimals);

        let breakdown = if !self.items.is_empty()
            || shipping_units > 0
            || tax_units > 0
            || discount_units > 0
        {
            Some(AmountBreakdown {
                item_total: if !self.items.is_empty() {
                    Some(Money::from_minor_units(item_total_units, self.currency))
                } else {
                    None
                },
                shipping: if shipping_units > 0 {
                    Some(Money::from_minor_units(shipping_units, self.currency))
                } else {
                    None
                },
                tax_total: if tax_units > 0 {
                    Some(Money::from_minor_units(tax_units, self.currency))
                } else {
                    None
                },
                discount: if discount_units > 0 {
                    Some(Money::from_minor_units(discount_units, self.currency))
                } else {
                    None
                },
                handling: None,
                insurance: None,
                shipping_discount: None,
            })
        } else {
            None
        };

        let final_units = if !self.items.is_empty() {
            item_total_units + shipping_units + tax_units - discount_units
        } else {
            Self::to_minor_units(self.amount, decimals)
        };

        let purchase_unit = PurchaseUnit {
            reference_id: None,
            description: self.description,
            custom_id: self.custom_id,
            invoice_id: self.invoice_id,
            soft_descriptor: self.soft_descriptor,
            amount: PurchaseAmount {
                currency_code: self.currency.to_string(),
                value: Money::format_minor_units(final_units, self.currency),
                breakdown,
            },
            items: if self.items.is_empty() {
                None
            } else {
                Some(self.items)
            },
            shipping: None,
            payments: None,
        };

        Ok(CreateOrderRequest {
            intent: self.intent,
            purchase_units: vec![purchase_unit],
            payment_source: None,
            application_context: Some(ApplicationContext {
                brand_name: self.brand_name,
                locale: self.locale,
                landing_page: Some(self.landing_page),
                shipping_preference: Some(self.shipping_preference),
                user_action: Some(self.user_action),
                return_url: self.return_url,
                cancel_url: self.cancel_url,
            }),
        })
    }

    fn compute_item_total_units(&self, decimals: u32) -> Result<i64> {
        Self::compute_items_total_units(&self.items, decimals)
    }

    fn compute_items_total_units(items: &[Item], decimals: u32) -> Result<i64> {
        let mut total: i64 = 0;
        let scale = 10_i64.pow(decimals);

        for item in items {
            let qty: u32 = item
                .quantity
                .parse()
                .map_err(|_| Error::Config("Invalid item quantity".into()))?;

            let price = item
                .unit_amount
                .amount()
                .map_err(|_| Error::Config("Invalid item amount".into()))?;

            let minor = (price * scale as f64).round() as i64;
            total += minor * qty as i64;
        }

        Ok(total)
    }

    fn to_minor_units(amount: f64, decimals: u32) -> i64 {
        let scale = 10_i64.pow(decimals);
        (amount * scale as f64).round() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(name: &str, qty: u32, unit_price: f64, currency: Currency) -> Item {
        Item {
            name: name.to_string(),
            quantity: qty.to_string(),
            unit_amount: Money::new(unit_price, currency),
            description: None,
            sku: None,
            category: None,
            tax: None,
        }
    }

    #[test]
    fn computes_item_total_minor_units() {
        let currency = Currency::USD;
        let decimals = currency.decimal_places() as u32;
        let items = vec![
            make_item("A", 2, 10.0, currency),
            make_item("B", 1, 5.5, currency),
        ];

        let total = OrderBuilder::compute_items_total_units(&items, decimals).unwrap();
        assert_eq!(total, 2550);
    }

    #[test]
    fn item_quantity_parse_error_surfaces() {
        let currency = Currency::USD;
        let decimals = currency.decimal_places() as u32;
        let mut item = make_item("A", 1, 10.0, currency);
        item.quantity = "bad".to_string();

        let err = OrderBuilder::compute_items_total_units(&[item], decimals).unwrap_err();
        match err {
            Error::Config(msg) => assert_eq!(msg, "Invalid item quantity"),
            _ => panic!("expected config error"),
        }
    }
}
