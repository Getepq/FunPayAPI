use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("не найден элемент с CSS-селектором: {0}")]
    SelectorNotFound(&'static str),

    #[error("у элемента отсутствует обязательный атрибут: {0}")]
    MissingAttribute(&'static str),

    #[error("обязательное поле пустое: {0}")]
    EmptyField(&'static str),

    #[error("не удалось разобрать JSON app_data: {0}")]
    InvalidAppData(#[source] serde_json::Error),
}
