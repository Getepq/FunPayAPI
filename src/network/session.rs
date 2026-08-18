use crate::parser::{Error as ParserError, Result as ParserResult, selectors::APP_DATA_SEL};
use scraper::Html;
use serde::Deserialize;

/// Промежуточная модель данных из атрибута `data-app-data`.
///
/// Тип используется только при разборе HTML. Токен защиты от межсайтовой
/// подделки запросов не включается в публичные модели библиотеки.
#[derive(Deserialize)]
struct RawAppData {
    #[serde(rename = "userId")]
    user_id: u32,

    #[serde(rename = "csrf-token")]
    csrf_token: String,
}

/// Данные, необходимые клиенту для работы с текущей сессией.
///
/// Токен защиты от межсайтовой подделки запросов остаётся приватным и
/// используется только внутренними сетевыми модулями.
pub(crate) struct SessionContext {
    #[expect(
        dead_code,
        reason = "Поле будет передаваться в запросы `POST`, требующие CSRF-защиты."
    )]
    csrf_token: String,
    user_id: u32,
}

impl SessionContext {
    /// Извлекает данные сессии из уже разобранного HTML-документа.
    ///
    /// Метод не выполняет сетевых запросов. Вызывающий код запускает его в
    /// `tokio::task::spawn_blocking`, а не в асинхронной задаче напрямую.
    pub(crate) fn from_document(document: &Html) -> ParserResult<Self> {
        let element = document
            .select(&APP_DATA_SEL)
            .next()
            .ok_or(ParserError::SelectorNotFound("body[data-app-data]"))?;

        let raw_app_data = element
            .value()
            .attr("data-app-data")
            .ok_or(ParserError::MissingAttribute("data-app-data"))?;

        let app_data = serde_json::from_str::<RawAppData>(raw_app_data)
            .map_err(ParserError::InvalidAppData)?;

        Ok(Self {
            csrf_token: app_data.csrf_token,
            user_id: app_data.user_id,
        })
    }

    /// Возвращает токен защиты от межсайтовой подделки запросов для внутреннего кода.
    #[expect(
        dead_code,
        reason = "Метод будет использован запросами `POST`, требующими CSRF-защиты."
    )]
    pub(crate) fn csrf_token(&self) -> &str {
        &self.csrf_token
    }

    /// Возвращает числовой идентификатор пользователя текущей сессии.
    pub(crate) fn user_id(&self) -> u32 {
        self.user_id
    }
}
