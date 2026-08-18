use serde::{Deserialize, Serialize};

/// Краткие сведения о предложении на `FunPay`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferPreview {
    /// Строковый идентификатор предложения.
    pub id: String,
    /// Отображаемое название предложения.
    pub name: String,
    /// Стоимость, указанная `FunPay`.
    pub price: String,
    /// Доступное количество товара.
    pub amount: u32,
    /// Лот, к которому относится предложение.
    pub lot: Lot,
}

/// Полные сведения о предложении на `FunPay`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    /// Строковый идентификатор предложения.
    pub id: String,
    /// Отображаемое название предложения.
    pub name: String,
    /// Категория предложения.
    pub category: String,
    /// Описание предложения, если оно указано продавцом.
    pub description: Option<String>,
    /// Стоимость, указанная `FunPay`.
    pub price: String,
    /// Доступное количество товара.
    pub amount: u32,
    /// Способ передачи товара, если он указан.
    pub method: Option<String>,
    /// Лот, к которому относится предложение.
    pub lot: Lot,
}

/// Лот `FunPay`, в рамках которого размещается предложение.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lot {
    /// Числовой идентификатор лота.
    pub id: i32,
    /// Наименование товара.
    pub product: String,
    /// Категория лота.
    pub category: String,
    /// Тип лота.
    pub r#type: LotTypes,
}

/// Тип лота в классификации `FunPay`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LotTypes {
    /// Обычный лот.
    Common,
    /// Лот с валютами.
    Chips,
}
