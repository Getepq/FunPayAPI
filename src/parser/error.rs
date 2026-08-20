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

    /// Значение не соответствует ожидаемому формату числового поля.
    #[error("Некорректное числовое значение поля {field}: {value}")]
    InvalidNumber {
        /// Имя разбираемого поля.
        field: &'static str,
        /// Исходное значение из HTML.
        value: String,
    },

    /// Ссылка `FunPay` не соответствует ожидаемому формату.
    #[error("Некорректная ссылка в поле {field}: {value}")]
    InvalidUrl {
        /// Имя разбираемого поля.
        field: &'static str,
        /// Исходное значение ссылки.
        value: String,
    },

    /// Страница детального предложения не соответствует переданному превью.
    #[error("Страница предложения не соответствует превью: ожидался {expected}, получен {actual}")]
    OfferMismatch {
        /// Ожидаемый тип и идентификатор предложения.
        expected: String,
        /// Тип и идентификатор, найденные в HTML.
        actual: String,
    },
}
