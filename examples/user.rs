use funpay_api::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализируем в проект аккаунт для работы с ним.
    let client = Client::new("", None).await?;

    // Получаем данные о пользователю, кому принадлежит golden_key.s
    let user = client.get_current_user().await?;

    // Выводим в консоль. Трейт Display не реализован для всех моделей.
    println!("{:?}", user);

    // Получаем данные о другом пользователя, в нашем примере это SidoRenko.
    let sidor_user = client.get_user(714925).await?;
    println!("{:?}", sidor_user);

    Ok(())
}
