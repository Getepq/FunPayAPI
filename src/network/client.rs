use crate::network::{ConfigError, Error, HttpError, NetworkError, Result};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use wreq::cookie::Jar;
use wreq::{Client, Method, Proxy, Response, Url, header::HeaderMap};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

static BASE_URL: LazyLock<Url> = LazyLock::new(move || {
    Url::parse("https://funpay.com/").expect("BASE_URL всегда имеет значение.")
});

pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    /// Функция создает экземпляр HTTP-клиента.
    /// 
    /// При создании, ваш golden_key помещается в "банку", которая к каждым запросам прикрепляет ваш golden_key.
    /// Помимо этого, она собирает те куки, которые отправил FunPay, это: PHPsessid и golden_seal.
    /// 
    /// HTTP-клиент, поддерживает работу через прокси таких типов: socks и http/https. 
    /// 
    /// Возвращает ошибки, в случаях: не передан golden_key (MissingGoldenKey), прокси не валиден (InvalidProxy), при сборке клиента произошла ошибка (ClientInit)
    pub fn new(golden_key: &str, proxy: Option<&str>) -> Result<Self> {
        if golden_key.is_empty() {
            return Err(Error::Config(ConfigError::MissingGoldenKey));
        }

        let jar = Jar::default();
        let cookies = format!("golden_key={}; Domain=funpay.com", golden_key);
        jar.add_cookie_str(&cookies, &BASE_URL);

        let emul_opt = EmulationOption::builder()
            .emulation_os(EmulationOS::Windows)
            .emulation(Emulation::Chrome137)
            .build();

        let mut buider = Client::builder()
            .emulation(emul_opt)
            .cookie_provider(Arc::new(jar))
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20));

        if let Some(proxy) = proxy {
            let proxy = Proxy::all(proxy).map_err(ConfigError::InvalidProxy)?;

            buider = buider.proxy(proxy)
        }

        let inner = buider.build().map_err(ConfigError::ClientInit)?;

        Ok(Self { inner })
    }

    /// Вспомогательная не публичная функция, которая собирает эндпоинт и обрабатывает ошибки.
    ///
    /// Поддерживает отправку POST-запросов с payload-ом.
    ///
    /// К запросам GET, POST также можно прикрепить кастомные загаловки, передав в параметры HeaderMap.
    /// 
    /// Возвращает ошибки в случаях: эндпоинт невалиден (InvalidEndpoint), сетевые ошибки (NetworkError), HTTP-статус не успешен (HttpError).
    async fn request<T: serde::Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        payload: Option<&T>,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        let endpoint = BASE_URL
            .join(path)
            .map_err(|e| ConfigError::InvalidEndpoint(e.to_string()))?;

        let mut req_builder = self.inner.request(method, endpoint);

        if let Some(payload) = payload {
            req_builder = req_builder.form(&payload);
        }

        if let Some(headers) = headers {
            req_builder = req_builder.headers(headers)
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                NetworkError::Timeout(e)
            } else if e.is_connect() {
                NetworkError::ConnectionFailed(e)
            } else if e.is_redirect() {
                NetworkError::TooManyRedirects(e)
            } else if e.is_builder() {
                NetworkError::RequestBuild(e)
            } else if e.is_connection_reset() {
                NetworkError::ConnectionReset(e)
            } else {
                NetworkError::Unknown(e)
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            let http_err = match status.as_u16() {
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
            return Err(Error::Http(http_err));
        }

        Ok(response)
    }

    /// Отправляет GET-запрос по указанному пути.
    ///
    /// Возвращает ошибки в случаях: эндпоинт невалиден (InvalidEndpoint), сетевые ошибки (NetworkError), HTTP-статус не успешен (HttpError).
    pub(crate) async fn get(&self, path: &str, headers: Option<HeaderMap>) -> Result<Response> {
        let response = self
            .request(Method::GET, path, None::<&()>, headers)
            .await?;
        Ok(response)
    }

    /// Отправляет POST-запрос по указанному пути с form-encoded payload-ом.
    ///
    /// Возвращает ошибки в случаях: эндпоинт невалиден (InvalidEndpoint), сетевые ошибки (NetworkError), HTTP-статус не успешен (HttpError).
    pub(crate) async fn post<T: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        payload: &T,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        let response = self
            .request(Method::POST, path, Some(payload), headers)
            .await?;
        Ok(response)
    }
}
