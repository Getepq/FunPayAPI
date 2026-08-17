use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] crate::network::Error),

    #[error(transparent)]
    Parser(#[from] crate::parser::Error),

    #[error("blocking-задача парсинга завершилась с ошибкой: {0}")]
    BlockingTask(String),
}
