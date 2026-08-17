use crate::parser::{Error as ParserError, Result as ParserResult, selectors::APP_DATA_SEL};
use scraper::Html;
use serde::Deserialize;

/// Сюда временно складываем JSON из `data-app-data`.
///
/// Наружу эту структуру не отдаём: внутри лежит csrf-токен, а ему нечего делать
/// в публичных моделях.
#[derive(Deserialize)]
struct RawAppData {
    #[serde(rename = "userId")]
    user_id: u32,

    #[serde(rename = "csrf-token")]
    csrf_token: String,
}

/// Данные, которые нужны самому клиенту для работы с текущей сессией.
///
/// Токен здесь лежит приватно и дальше network-кода не уезжает.
pub(crate) struct SessionContext {
    csrf_token: String,
    user_id: u32,
}

impl SessionContext {
    /// Достаём session-данные из уже распарсенного HTML.
    ///
    /// Функция sync, поэтому вызывающий код должен запускать её через
    /// `spawn_blocking`, а не прямо внутри async flow.
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

    /// Даём токен только внутреннему network-коду.
    pub(crate) fn csrf_token(&self) -> &str {
        &self.csrf_token
    }

    /// ID текущего пользователя без всяких секретов рядом.
    pub(crate) fn user_id(&self) -> u32 {
        self.user_id
    }
}
