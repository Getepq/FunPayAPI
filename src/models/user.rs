use crate::models::Balances;
use serde::{Deserialize, Serialize};

/// Профиль пользователя FunPay.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// Числовой ID профиля.
    pub id: u32,
    /// Отображаемый username.
    pub username: String,
    /// Абсолютный URL аватарки или `None`, если inline style отсутствует.
    pub avatar_url: Option<String>,
    /// Текущий статус профиля.
    pub status: Status,
    /// Количество отзывов, если FunPay отдал rating block.
    pub reviews_count: Option<u32>,
    /// Абсолютная дата регистрации без подсказки.
    pub registered_at: String,
}

/// Профиль текущего пользователя вместе с балансами.
///
/// Пока парсинг баланса не написан, `Client::get_current_user` возвращает
/// только `User`; этот тип зарезервирован под полный current-user flow.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    /// Профиль текущего пользователя.
    pub user: User,
    /// Балансы по поддерживаемым валютам.
    pub balance: Balances,
}

/// Статус пользователя в профиле.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Пользователь сейчас онлайн.
    Online,
    /// Пользователь оффлайн с текстом последнего визита.
    Offline { last_seen: String },
    /// Профиль заблокирован.
    Blocked,
}
