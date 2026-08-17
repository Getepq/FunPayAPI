//! Async HTTP-транспорт для FunPay.
//!
//! Этот модуль отвечает только за I/O: куки, прокси, таймауты, HTTP-статусы
//! и чтение response body. HTML-парсинг живёт выше и запускается через
//! `tokio::task::spawn_blocking`.

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

/// Внутренний HTTP-клиент с "банкой" куков и транспорт-настройками.
pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    /// Создаёт транспорт-клиент с "банкой" куков и опциональным прокси.
    ///
    /// `golden_key` кладётся только во внутреннию "банку" кук. В публичные
    /// поля, ошибки и debug-output секрет не прокидывается.
    ///
    /// Поддерживаются HTTP/HTTPS- и SOCKS-прокси. Ошибки конфигурации
    /// возвращаются typed-ами, без паники на пользовательском вводе.
    pub fn new(golden_key: &str, proxy: Option<&str>) -> Result<Self> {
        if golden_key.is_empty() {
            return Err(Error::Config(ConfigError::MissingGoldenKey));
        }

        let jar = Jar::default();
        let cookies = format!("golden_key={}; Domain=funpay.com", golden_key);
        jar.add_cookie_str(&cookies, &BASE_URL);

        let emulation = EmulationOption::builder()
            .emulation_os(EmulationOS::Windows)
            .emulation(Emulation::Chrome137)
            .build();

        let mut builder = Client::builder()
            .emulation(emulation)
            .cookie_provider(Arc::new(jar))
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20));

        if let Some(proxy) = proxy {
            let proxy = Proxy::all(proxy).map_err(ConfigError::InvalidProxy)?;
            builder = builder.proxy(proxy);
        }

        let inner = builder.build().map_err(ConfigError::ClientInit)?;

        Ok(Self { inner })
    }

    /// Собирает эндпоинт, отправляет запрос и мапит транспортные-ошибки.
    ///
    /// Внутренний request-builder поддерживает GET/POST и опциональные загаловки.
    /// Неуспешные HTTP-статусы превращаются в типизированные `HttpError`, чтобы
    /// вызывающий код мог делать нормальный match, а не парсить строки.
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

    /// Асинхронно выполняет GET-запрос по относительному FunPay пути.
    pub(crate) async fn get(&self, path: &str, headers: Option<HeaderMap>) -> Result<Response> {
        self.request(Method::GET, path, None::<&()>, headers).await
    }

    /// Асинхронно получает HTML тело.
    ///
    /// Здесь нет DOM-парсинга: метод только читает тело и отдаёт
    /// owned `String` в слой парсинга.
    pub(crate) async fn get_html(&self, path: &str) -> Result<String> {
        let response = self.get(path, None).await?;
        response.text().await.map_err(Error::ResponseRead)
    }

    /// Асинхронно отправляет form-encoded POST-запрос.
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
