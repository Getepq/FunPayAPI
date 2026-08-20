//! Асинхронный HTTP-клиент для `FunPay`.
//!
//! Модуль отвечает за cookie, прокси-серверы, ограничения времени ожидания,
//! обработку HTTP-статусов и чтение тела ответа. Разбор HTML выполняется
//! вышестоящими модулями через `tokio::task::spawn_blocking`.

use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use crate::network::{ConfigError, Error, HttpError, NetworkError, Result};
use wreq::cookie::Jar;
use wreq::{Client, Method, Proxy, Response, Url, header::HeaderMap};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

static BASE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://funpay.com/").expect("BASE_URL всегда имеет значение."));

/// Внутренний HTTP-клиент с хранилищем cookie и настройками транспорта.
pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    /// Создаёт HTTP-клиент с cookie `golden_key` и необязательным прокси-сервером.
    ///
    /// Ключ сохраняется только во внутреннем хранилище cookie и не передаётся
    /// через публичные модели, ошибки или журнал событий. Поддерживаются
    /// HTTP-, HTTPS- и SOCKS-прокси-серверы.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку, если `golden_key` пуст, адрес прокси-сервера
    /// некорректен или не удалось создать HTTP-клиент.
    pub fn new(golden_key: &str, proxy: Option<&str>) -> Result<Self> {
        if golden_key.is_empty() {
            return Err(Error::Config(ConfigError::MissingGoldenKey));
        }

        let jar = Jar::default();
        let cookies = format!("golden_key={golden_key}; Domain=funpay.com");
        jar.add_cookie_str(&cookies, &BASE_URL);

        let emulation = EmulationOption::builder()
            .emulation_os(EmulationOS::Windows)
            .emulation(Emulation::Chrome137)
            .build();

        let mut builder = Client::builder()
            .emulation(emulation)
            .cookie_provider(Arc::new(jar))
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(20))
            .timeout(Duration::from_secs(30));

        if let Some(proxy) = proxy {
            let proxy = Proxy::all(proxy).map_err(ConfigError::InvalidProxy)?;
            builder = builder.proxy(proxy);
        }

        let inner = builder.build().map_err(ConfigError::ClientInit)?;

        Ok(Self { inner })
    }

    /// Формирует и отправляет запрос к относительному пути `FunPay`.
    ///
    /// Метод поддерживает запросы `GET` и `POST`, данные формы и дополнительные
    /// заголовки. Неуспешные HTTP-статусы преобразуются в [`HttpError`], чтобы
    /// вызывающий код сопоставлял варианты ошибок, а не разбирал текст сообщений.
    async fn request<T: serde::Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        payload: Option<&T>,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        let endpoint = BASE_URL
            .join(path)
            .map_err(|error| ConfigError::InvalidEndpoint(error.to_string()))?;

        let mut request = self.inner.request(method, endpoint);

        if let Some(payload) = payload {
            request = request.form(payload);
        }

        if let Some(headers) = headers {
            request = request.headers(headers);
        }

        let response = request.send().await.map_err(|error| {
            if error.is_timeout() {
                NetworkError::Timeout(error)
            } else if error.is_connect() {
                NetworkError::ConnectionFailed(error)
            } else if error.is_redirect() {
                NetworkError::TooManyRedirects(error)
            } else if error.is_builder() {
                NetworkError::RequestBuild(error)
            } else if error.is_connection_reset() {
                NetworkError::ConnectionReset(error)
            } else {
                NetworkError::Unknown(error)
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            let http_error = match status.as_u16() {
                400 => HttpError::BadRequest,
                401 => HttpError::Unauthorized,
                403 => HttpError::Forbidden,
                404 => HttpError::NotFound,
                429 => HttpError::TooManyRequests,
                500 => HttpError::InternalServerError,
                502 => HttpError::BadGateway,
                503 => HttpError::ServiceUnavailable,
                504 => HttpError::GatewayTimeout,
                code => HttpError::Other(code),
            };

            return Err(Error::Http(http_error));
        }

        Ok(response)
    }

    /// Выполняет запрос `GET` по относительному пути `FunPay`.
    pub(crate) async fn get(&self, path: &str, headers: Option<HeaderMap>) -> Result<Response> {
        self.request(Method::GET, path, None::<&()>, headers).await
    }

    /// Отправляет запрос `POST` с данными формы по относительному пути `FunPay`.
    #[expect(
        dead_code,
        reason = "Метод будет использован при создании и изменении сущностей `FunPay`."
    )]
    pub(crate) async fn post<T: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        payload: &T,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        self.request(Method::POST, path, Some(payload), headers)
            .await
    }
}
