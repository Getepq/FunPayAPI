use crate::error::{Error, Result};
use crate::models::{Balances, CurrentUser, LotTypes, Offer, OfferPreview, User};
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

    /// Разбирает HTML страницы профиля и возвращает все превью предложений.
    ///
    /// Разбор запускается через `tokio::task::spawn_blocking`, чтобы не
    /// блокировать исполнитель Tokio обработкой HTML.
    async fn parse_offer_previews_html(html: String) -> Result<Vec<OfferPreview>> {
        let parsed = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            parser::get_offer_previews(&document)
        })
        .await
        .map_err(|error| Error::BlockingTask(error.to_string()))?;

        parsed.map_err(Error::from)
    }

    /// Разбирает HTML детальной страницы и объединяет его с превью предложения.
    ///
    /// Идентификатор и тип страницы сверяются внутри парсера с `preview`.
    async fn parse_offer_html(html: String, preview: OfferPreview) -> Result<Offer> {
        let parsed = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            parser::get_offer(&document, preview)
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

    /// Загружает все отображаемые превью предложений указанного пользователя.
    ///
    /// Метод запрашивает публичную страницу профиля `/users/<id>/` и извлекает
    /// строки таблиц лотов, включая обычные лоты и Chips.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку при неуспешном HTTP-запросе, невозможности прочитать
    /// ответ, разобрать карточки предложений или выполнить задачу с
    /// блокирующей операцией.
    pub async fn get_user_offers(&self, user_id: u32) -> Result<Vec<OfferPreview>> {
        let path = format!("/users/{user_id}/");
        let html = self.fetch_html(&path).await?;

        Self::parse_offer_previews_html(html).await
    }

    /// Загружает и разбирает детальную страницу предложения из ранее полученного превью.
    ///
    /// Путь выбирается по [`LotTypes`], а страница дополнительно сверяется с
    /// типом и идентификатором `preview` перед созданием [`Offer`].
    ///
    /// # Errors
    ///
    /// Возвращает ошибку при неуспешном HTTP-запросе, невозможности прочитать
    /// ответ, разобрать страницу или выполнить задачу с блокирующей операцией.
    pub async fn get_offer(&self, preview: OfferPreview) -> Result<Offer> {
        let path = match &preview.lot.r#type {
            LotTypes::Common => format!("/lots/offer?id={}", preview.id),
            LotTypes::Chips => format!("/chips/offer?id={}", preview.id),
        };
        let html = self.fetch_html(&path).await?;

        Self::parse_offer_html(html, preview).await
    }

    /// Загружает все отображаемые превью предложений пользователя текущей сессии.
    ///
    /// Метод использует идентификатор, полученный при создании [`Client`], и
    /// эквивалентен вызову [`Client::get_user_offers`] для этого пользователя.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку при неуспешном HTTP-запросе, невозможности прочитать
    /// ответ, разобрать карточки предложений или выполнить задачу с
    /// блокирующей операцией.
    pub async fn get_current_offers(&self) -> Result<Vec<OfferPreview>> {
        self.get_user_offers(self.session.user_id()).await
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
