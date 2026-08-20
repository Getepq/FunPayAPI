use funpay_api::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Создаём клиента для работы с учётной записью.
    let client = Client::new("", None).await?;

    // Получаем все оффера владельца ключа.
    let offers = client.get_current_offers().await?;
    println!("{:?}", offers[0]);
    

    // Получаем полную модель первого offer-а.
    let offer = client.get_offer(offers[0].clone()).await?;
    println!("{offer:?}");

    Ok(())
}
