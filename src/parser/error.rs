use thiserror::Error;

/// Результат выполнения внутренних операций разбора HTML.
pub type Result<T> = std::result::Result<T, Error>;

/// Ошибки извлечения данных из HTML-страниц `FunPay`.
#[derive(Debug, Error)]
pub enum Error {
    /// Не найден обязательный элемент по CSS-селектору.
    #[error("Не найден элемент с CSS-селектором: {0}")]
    SelectorNotFound(&'static str),

    /// У элемента отсутствует обязательный атрибут.
    #[error("У элемента отсутствует обязательный атрибут: {0}")]
    MissingAttribute(&'static str),

    /// Обязательное текстовое значение отсутствует или пусто.
    #[error("Обязательное поле пустое: {0}")]
    EmptyField(&'static str),

    /// Значение атрибута `data-app-data` содержит некорректный JSON.
    #[error("Не удалось разобрать JSON app_data: {0}")]
    InvalidAppData(#[source] serde_json::Error),
}
