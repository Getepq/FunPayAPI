use serde::{Deserialize, Serialize};

/// Краткие сведения о чате `FunPay`.
///
/// Поля будут добавлены при реализации разбора списка чатов.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatPreview {}

/// Полные сведения о чате `FunPay`.
///
/// Поля будут добавлены при реализации разбора страницы чата.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chat {}

/// Сообщение в чате `FunPay`.
///
/// Поля будут добавлены при реализации разбора сообщений.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {}

/// Отправитель сообщения.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgFrom {
    /// Сообщение отправлено пользователем.
    User,
    /// Сообщение сформировано системой `FunPay`.
    System,
}

/// Вид сообщения в чате.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgTypes {
    /// Текстовое сообщение.
    Text,
    /// Сообщение с изображением.
    Image,
}
