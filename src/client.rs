use crate::error::{Error, Result};
use crate::models::{Balances, CurrentUser, User};
use crate::network::Error::ResponseRead;
use crate::network::{HttpClient, SessionContext};
use crate::parser;
use scraper::Html;
use tokio::task::spawn_blocking;

/// Основной клиент для работы с учётной записью `FunPay`.
///
/// Клиент хранит HTTP-клиент и контекст текущей сессии. Секретные данные
/// остаются во внутренних типах и не входят в публичные модели или ошибки.
pub struct Client {
    inner: HttpClient,
    session: SessionContext,
}

impl Client {
    /// Создаёт клиент для учётной записи `FunPay`.
    ///
    /// Переданный `golden_key` сохраняется только во внутреннем хранилище cookie.
    /// Если `proxy` равен `None`, запросы отправляются без прокси-сервера.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку при некорректном ключе или настройках прокси-сервера,
    /// невозможности создать HTTP-клиент, загрузить начальную страницу,
    /// разобрать данные сессии либо выполнить задачу с блокирующей операцией.
    pub async fn new(golden_key: &str, proxy: Option<&str>) -> Result<Self> {
        let inner = HttpClient::new(golden_key, proxy)?;
        let session = Self::initialize_session(&inner).await?;

        Ok(Self { inner, session })
    }

    /// Загружает исходный HTML по относительному пути `FunPay`.
    ///
    /// Метод отвечает только за отправку запроса и чтение тела ответа. Разбор
    /// HTML выполняется вызывающим методом в отдельной задаче с блокирующей
    /// операцией.
    async fn fetch_html(&self, path: &str) -> Result<String> {
        let resposnse = self.inner.get(path, None).await?;
        let html = resposnse
            .text()
            .await
            .map_err(|e| Error::Network(ResponseRead(e)))?;
        Ok(html)
    }

    /// Разбирает HTML страницы профиля и возвращает модель пользователя.
    ///
    /// Разбор HTML не выполняет сетевых запросов. Он запускается через
    /// `tokio::task::spawn_blocking`, чтобы не блокировать исполнитель Tokio.
    async fn parse_user_html(&self, html: String, user_id: u32) -> Result<User> {
        let parsed = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            parser::get_user(&document, user_id)
        })
        .await
        .map_err(|error| Error::BlockingTask(error.to_string()))?;

        parsed.map_err(Error::from)
    }

    /// Разбирает HTML страницы баланса и возвращает значения по валютам.
    ///
    /// Разбор HTML выполняется через `tokio::task::spawn_blocking`, чтобы не
    /// блокировать исполнитель Tokio.
    async fn parse_balances_html(html: String) -> Result<Balances> {
        let parsed = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            parser::get_balances(&document)
        })
        .await
        .map_err(|error| Error::BlockingTask(error.to_string()))?;

        parsed.map_err(Error::from)
    }

    /// Извлекает данные текущей сессии из начальной страницы `FunPay`.
    ///
    /// Токен защиты от межсайтовой подделки запросов сохраняется только в
    /// `SessionContext` и не передаётся через публичный интерфейс.
    async fn initialize_session(inner: &HttpClient) -> Result<SessionContext> {
        let response = inner.get("/", None).await?;
        let html = response
            .text()
            .await
            .map_err(|e| Error::Network(ResponseRead(e)))?;

        let parsed = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            SessionContext::from_document(&document)
        })
        .await
        .map_err(|error| Error::BlockingTask(error.to_string()))?;

        parsed.map_err(Error::from)
    }

    /// Загружает и разбирает профиль пользователя по его числовому идентификатору.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку при неуспешном HTTP-запросе, невозможности прочитать
    /// ответ, разобрать HTML профиля или выполнить задачу с блокирующей
    /// операцией.
    pub async fn get_user(&self, user_id: u32) -> Result<User> {
        let path = format!("/users/{user_id}/");
        let html = self.fetch_html(&path).await?;

        self.parse_user_html(html, user_id).await
    }

    /// Загружает профиль и балансы пользователя, которому принадлежит `golden_key`.
    ///
    /// Профиль загружается со страницы пользователя, а балансы — со страницы
    /// `/account/balance`.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку при неуспешном HTTP-запросе, невозможности прочитать
    /// ответ, разобрать HTML профиля или баланса либо выполнить задачу с
    /// блокирующей операцией.
    pub async fn get_current_user(&self) -> Result<CurrentUser> {
        let user = self.get_user(self.session.user_id()).await?;

        let balance_html = self.fetch_html("/account/balance").await?;
        let balance = Self::parse_balances_html(balance_html).await?;

        Ok(CurrentUser { user, balance })
    }
}
