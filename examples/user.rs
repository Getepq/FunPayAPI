use funpay_api::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Создаём клиента для работы с учётной записью.
    let client = Client::new("", None).await?;

    // Загружаем профиль и балансы владельца ключа.
    let current_user = client.get_current_user().await?;
    println!("{current_user:?}");

    // Загружаем профиль другого пользователя по его идентификатору.
    let user = client.get_user(714_925).await?;
    println!("{user:?}");

    Ok(())
}
