use serde::{Deserialize, Serialize};

/// Краткие сведения о предложении на `FunPay`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferPreview {
    /// Строковый идентификатор предложения.
    pub id: String,
    /// Краткое описание предложения, используемое как его отображаемое имя.
    pub name: String,
    /// Стоимость, указанная одной еденицы товара.
    pub price: String,
    /// Доступное количество товара.
    pub amount: OfferAmount,
    /// Лот, к которому относится предложение.
    pub lot: Lot,
}

/// Одно произвольное поле из блока параметров предложения.
///
/// `FunPay` может добавлять новые поля, менять их порядок или использовать
/// различные названия для одного и того же смысла. Поэтому исходные пары
/// `название`/`значение` сохраняются без потери информации.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferField {
    /// Заголовок, отображённый в `h5` внутри `.param-item`.
    pub name: String,
    /// Текстовое значение поля.
    pub value: String,
}

/// Полные сведения о предложении на `FunPay`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    /// Строковый идентификатор предложения.
    pub id: String,
    /// Краткое описание предложения, используемое как его отображаемое имя.
    pub name: String,
    /// Значение поля типа предложения, если оно указано на странице.
    pub offer_type: Option<String>,
    /// Подробное описание предложения, если оно указано продавцом.
    pub description: Option<String>,
    /// Стоимость, указанная одной еденицы товара.
    pub price: String,
    /// Доступное количество товара или его исходное текстовое значение.
    pub amount: OfferAmount,
    /// Способ получения или передачи товара, если он указан.
    pub delivery_method: Option<String>,
    /// Все поля `.param-item` в порядке, в котором они показаны на странице.
    ///
    /// Это поле содержит также стандартные параметры из `name`, `description`,
    /// `amount` и `delivery_method`, если они присутствуют в HTML.
    pub fields: Vec<OfferField>,
    /// Лот, к которому относится предложение.
    pub lot: Lot,
}

impl Offer {
    /// Возвращает значение произвольного поля по его названию.
    ///
    /// Сравнение выполняется без учёта регистра и повторяющихся пробелов.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| normalize_field_name(&field.name) == normalize_field_name(name))
            .map(|field| field.value.as_str())
    }
}

/// Нормализует названия поля перед сравнением.
fn normalize_field_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

impl OfferField {
    /// Создает поле предложения, а из его загловка - значение.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Доступное количество товара в предложение.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferAmount {
    /// Конечное количество товара.
    Quantity(u32),
    /// Исходное значение, которое нельзя безопасно преобразовать в число.
    Raw(String),
}

/// Лот `FunPay`, в рамках которого размещается предложение.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LotTypes {
    /// Обычный лот.
    Common,
    /// Лот с валютами.
    Chips,
}
