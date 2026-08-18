use serde::{Deserialize, Serialize};

/// Краткие сведения о заказе на `FunPay`.
///
/// Поля будут добавлены при реализации разбора списка заказов.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderPreview {}

/// Полные сведения о заказе на `FunPay`.
///
/// Поля будут добавлены при реализации разбора страницы заказа.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {}
