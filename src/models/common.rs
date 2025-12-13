use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    TRY,
    JPY,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::TRY => "TRY",
            Currency::JPY => "JPY",
        }
    }

    pub fn decimal_places(&self) -> u8 {
        if matches!(self, Currency::JPY) {
            0
        } else {
            2
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub currency_code: String,
    pub value: String,
}

impl Money {
    pub fn new(amount: f64, currency: Currency) -> Self {
        let value = Self::format(amount, currency);
        Self {
            currency_code: currency.to_string(),
            value,
        }
    }

    pub fn from_minor_units(units: i64, currency: Currency) -> Self {
        let value = Self::format_minor_units(units, currency);
        Self {
            currency_code: currency.to_string(),
            value,
        }
    }

    pub fn format(amount: f64, currency: Currency) -> String {
        let decimals = currency.decimal_places() as usize;
        if decimals == 0 {
            format!("{:.0}", amount)
        } else {
            format!("{:.*}", decimals, amount)
        }
    }

    pub fn format_minor_units(units: i64, currency: Currency) -> String {
        let decimals = currency.decimal_places() as u32;
        if decimals == 0 {
            return units.to_string();
        }

        let scale = 10_i64.pow(decimals);
        let integer = units / scale;
        let fraction = (units.abs() % scale) as u64;
        format!("{integer}.{fraction:0width$}", width = decimals as usize)
    }

    pub fn amount(&self) -> Result<f64, ParseFloatError> {
        self.value.parse()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub href: String,
    pub rel: String,
    pub method: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Intent {
    #[default]
    Capture,
    Authorize,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LandingPage {
    #[default]
    Login,
    GuestCheckout,
    NoPreference,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserAction {
    #[default]
    Continue,
    PayNow,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShippingPreference {
    #[default]
    GetFromFile,
    NoShipping,
    SetProvidedAddress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_respects_zero_decimal_currencies() {
        let money = Money::new(1234.0, Currency::JPY);
        assert_eq!(money.value, "1234");
    }

    #[test]
    fn money_respects_two_decimal_currencies() {
        let money = Money::new(10.5, Currency::USD);
        assert_eq!(money.value, "10.50");
    }

    #[test]
    fn format_minor_units_supports_zero_and_two_decimals() {
        let usd = Money::from_minor_units(1234, Currency::USD);
        assert_eq!(usd.value, "12.34");

        let jpy = Money::from_minor_units(987, Currency::JPY);
        assert_eq!(jpy.value, "987");
    }

    #[test]
    fn amount_propagates_parse_error() {
        let money = Money {
            currency_code: "USD".into(),
            value: "oops".into(),
        };

        assert!(money.amount().is_err());
    }
}
