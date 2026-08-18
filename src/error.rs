use thiserror::Error;

/// Результат выполнения публичных операций библиотеки.
pub type Result<T> = std::result::Result<T, Error>;

/// Ошибки, возвращаемые публичным интерфейсом библиотеки.
#[derive(Debug, Error)]
pub enum Error {
    /// Ошибка настройки или выполнения сетевого запроса.
    #[error(transparent)]
    Network(#[from] crate::network::Error),

    /// Ошибка разбора HTML-страницы `FunPay`.
    #[error(transparent)]
    Parser(#[from] crate::parser::Error),

    /// Ошибка выполнения задачи с блокирующей операцией.
    #[error("Задача разбора с блокирующей операцией завершилась с ошибкой: {0}")]
    BlockingTask(String),
}
