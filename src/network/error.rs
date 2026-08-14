use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Http(#[from] HttpError),

    #[error("Не удалось прочитать тело ответа: {0}")]
    ResponseRead(#[source] wreq::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Не указан обязательный параметр: golden_key")]
    MissingGoldenKey,

    #[error("Некорректный адрес прокси-сервера: {0}")]
    InvalidProxy(#[source] wreq::Error),

    #[error("Ошибка при инициализации HTTP-клиента: {0}")]
    ClientInit(#[source] wreq::Error),

    #[error("Некорректный путь эндпоинта: {0}")]
    InvalidEndpoint(String),
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Превышено время ожидания ответа от сервера")]
    Timeout(#[source] wreq::Error),

    #[error("Не удалось установить соединение с сервером")]
    ConnectionFailed(#[source] wreq::Error),

    #[error("Соединение было разорвано сервером")]
    ConnectionReset(#[source] wreq::Error),

    #[error("Превышено количество перенаправлений (redirects)")]
    TooManyRedirects(#[source] wreq::Error),

    #[error("Внутренняя ошибка при формировании запроса")]
    RequestBuild(#[source] wreq::Error),

    #[error("Неизвестная сетевая ошибка")]
    Unknown(#[source] wreq::Error),
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Некорректный запрос (400 Bad Request)")]
    BadRequest,

    #[error("Ошибка авторизации. Возможно, golden_key недействителен (401 Unauthorized)")]
    Unauthorized,

    #[error("Доступ запрещен (403 Forbidden)")]
    Forbidden,

    #[error("Ресурс не найден (404 Not Found)")]
    NotFound,

    #[error("Слишком много запросов (429 Too Many Requests)")]
    TooManyRequests,

    #[error("Внутренняя ошибка сервера (500 Internal Server Error)")]
    InternalServerError,

    #[error("Неверный шлюз (502 Bad Gateway)")]
    BadGateway,

    #[error("Сервис недоступен (503 Service Unavailable)")]
    ServiceUnavailable,

    #[error("Время ожидания шлюза истекло (504 Gateway Timeout)")]
    GatewayTimeout,

    #[error("Специфичная ошибка HTTP: {0}")]
    Other(u16),
}
