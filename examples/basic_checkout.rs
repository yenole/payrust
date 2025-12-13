use payrust::prelude::*;

#[tokio::main]
async fn main() -> payrust::Result<()> {
    dotenvy::dotenv().ok();

    let client_id = std::env::var("PAYPAL_CLIENT_ID").expect("PAYPAL_CLIENT_ID not set");
    let client_secret =
        std::env::var("PAYPAL_CLIENT_SECRET").expect("PAYPAL_CLIENT_SECRET not set");

    let client = PayPal::sandbox(&client_id, &client_secret).await?;

    let order = client
        .create_order()
        .amount(29.99, Currency::USD)
        .item("Premium Membership", 1, 29.99)
        .description("Monthly Premium Membership")
        .custom_id("order-12345")
        .return_url("https://example.com/success")
        .cancel_url("https://example.com/cancel")
        .send()
        .await?;

    println!("Order ID: {}", order.id);
    println!("Status: {:?}", order.status);
    if let Some(url) = order.approve_url() {
        println!("Approve: {}", url);
    }

    Ok(())
}
