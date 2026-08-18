use thiserror::Error;

/// Результат выполнения внутренних сетевых операций.
pub type Result<T> = std::result::Result<T, Error>;

/// Ошибки настройки и выполнения сетевых запросов.
#[derive(Debug, Error)]
pub enum Error {
    /// Ошибка настройки HTTP-клиента.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Ошибка сетевого соединения.
    #[error(transparent)]
    Network(#[from] NetworkError),

    /// Неуспешный HTTP-статус ответа.
    #[error(transparent)]
    Http(#[from] HttpError),

    /// Ошибка чтения тела HTTP-ответа.
    #[error("Не удалось прочитать тело ответа: {0}")]
    ResponseRead(#[source] wreq::Error),
}

/// Ошибки настройки HTTP-клиента.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Не указан обязательный ключ `golden_key`.
    #[error("Не указан обязательный параметр: golden_key")]
    MissingGoldenKey,

    /// Указан некорректный адрес прокси-сервера.
    #[error("Некорректный адрес прокси-сервера: {0}")]
    InvalidProxy(#[source] wreq::Error),

    /// Не удалось создать HTTP-клиент.
    #[error("Ошибка при инициализации HTTP-клиента: {0}")]
    ClientInit(#[source] wreq::Error),

    /// Относительный путь нельзя преобразовать в адрес `FunPay`.
    #[error("Некорректный путь эндпоинта: {0}")]
    InvalidEndpoint(String),
}

/// Ошибки, возникшие до получения HTTP-ответа.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Истекло время ожидания ответа.
    #[error("Превышено время ожидания ответа от сервера")]
    Timeout(#[source] wreq::Error),

    /// Не удалось установить соединение с сервером.
    #[error("Не удалось установить соединение с сервером")]
    ConnectionFailed(#[source] wreq::Error),

    /// Сервер разорвал установленное соединение.
    #[error("Соединение было разорвано сервером")]
    ConnectionReset(#[source] wreq::Error),

    /// Превышено допустимое количество перенаправлений.
    #[error("Превышено количество перенаправлений (redirects)")]
    TooManyRedirects(#[source] wreq::Error),

    /// Не удалось сформировать запрос.
    #[error("Внутренняя ошибка при формировании запроса")]
    RequestBuild(#[source] wreq::Error),

    /// Возникла сетевая ошибка, которую не удалось отнести к известной категории.
    #[error("Неизвестная сетевая ошибка")]
    Unknown(#[source] wreq::Error),
}

/// Неуспешные HTTP-статусы, возвращённые `FunPay`.
#[derive(Debug, Error)]
pub enum HttpError {
    /// Некорректный запрос.
    #[error("Некорректный запрос (400 Bad Request)")]
    BadRequest,

    /// Ошибка авторизации; возможно, `golden_key` недействителен.
    #[error("Ошибка авторизации. Возможно, golden_key недействителен (401 Unauthorized)")]
    Unauthorized,

    /// Доступ к ресурсу запрещён.
    #[error("Доступ запрещен (403 Forbidden)")]
    Forbidden,

    /// Запрошенный ресурс не найден.
    #[error("Ресурс не найден (404 Not Found)")]
    NotFound,

    /// Превышено допустимое число запросов.
    #[error("Слишком много запросов (429 Too Many Requests)")]
    TooManyRequests,

    /// Внутренняя ошибка сервера.
    #[error("Внутренняя ошибка сервера (500 Internal Server Error)")]
    InternalServerError,

    /// Промежуточный сервер вернул неверный ответ.
    #[error("Неверный шлюз (502 Bad Gateway)")]
    BadGateway,

    /// Сервис временно недоступен.
    #[error("Сервис недоступен (503 Service Unavailable)")]
    ServiceUnavailable,

    /// Промежуточный сервер не дождался ответа.
    #[error("Время ожидания шлюза истекло (504 Gateway Timeout)")]
    GatewayTimeout,

    /// HTTP-статус, для которого нет отдельного варианта.
    #[error("Специфичная ошибка HTTP: {0}")]
    Other(u16),
}
