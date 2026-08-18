use crate::models::Balances;
use serde::{Deserialize, Serialize};

/// Профиль пользователя `FunPay`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// Числовой идентификатор профиля.
    pub id: u32,
    /// Отображаемое имя пользователя.
    pub username: String,
    /// Абсолютный адрес изображения профиля.
    pub avatar_url: String,
    /// Текущее состояние пользователя.
    pub status: Status,
    /// Количество отзывов, если `FunPay` отобразил соответствующий блок.
    pub reviews_count: Option<u32>,
    /// Дата регистрации без относительного описания давности.
    pub registered_at: String,
}

/// Данные учётной записи, которой принадлежит `golden_key`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    /// Профиль пользователя.
    pub user: User,
    /// Балансы по поддерживаемым валютам.
    pub balance: Balances,
}

/// Состояние пользователя в профиле.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Пользователь находится в сети.
    Online,
    /// Пользователь не в сети; указано время последнего посещения.
    Offline {
        /// Текст времени последнего посещения, отображённый `FunPay`.
        last_seen: String,
    },
    /// Профиль заблокирован.
    Blocked,
}
