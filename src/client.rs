use crate::error::{Error, Result};
use crate::models::User;
use crate::network::{HttpClient, SessionContext};
use crate::parser;
use scraper::Html;
use tokio::task::spawn_blocking;

/// Главный объект, через который управляем аккаунтом в FunPay.
///
/// Внутри лежат HTTP-клиент и session-контекст. Токены и куки наружу
/// не торчат - наружу отдаём только модели и ошибки.
pub struct Client {
    inner: HttpClient,
    session: Option<SessionContext>,
}

impl Client {
    /// Создаёт клиента и кладёт `golden_key` во внутреннию "банку" с куками.
    ///
    /// Ключ не возвращается, не логируется и не торчит в публичных полях.
    /// Прокси можно не указывать - тогда запросы идут напрямую.
    pub fn new(golden_key: &str, proxy: Option<&str>) -> Result<Self> {
        Ok(Self {
            inner: HttpClient::new(golden_key, proxy)?,
            session: None,
        })
    }

    /// Забирает HTML. Тут только сеть и чтение body, без парсинга.
    ///
    /// DOM собираем позже, уже в blocking-задаче, чтобы не стопить рантайм.
    async fn fetch_html(&self, path: &str) -> Result<String> {
        self.inner.get_html(path).await.map_err(Error::from)
    }

    /// Отдаёт HTML в `spawn_blocking` и возвращает готового `User`.
    ///
    /// `scraper` синхронный и CPU-bound. Если запускать его прямо внутри async,
    /// можно случайно подвесить рантайм. Поэтому здесь весь DOM flow живёт
    /// в отдельном blocking worker-е.
    async fn parse_user_html(&self, html: String, user_id: u32) -> Result<User> {
        let parsed = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            parser::get_user(&document, user_id)
        })
        .await
        .map_err(|error| Error::BlockingTask(error.to_string()))?;

        parsed.map_err(Error::from)
    }

    /// Достаёт `user_id` из главной страницы и кеширует session-контекст.
    ///
    /// `csrf_token` остаётся внутри `SessionContext`. Наружу уходит только
    /// ID пользователя, сам секрет по проекту не "гуляет".
    async fn initialize_session(&mut self) -> Result<u32> {
        if let Some(session) = self.session.as_ref() {
            return Ok(session.user_id());
        }

        let html = self.fetch_html("/").await?;

        let session = spawn_blocking(move || {
            let document = Html::parse_document(&html);
            SessionContext::from_document(&document)
        })
        .await
        .map_err(|error| Error::BlockingTask(error.to_string()))?
        .map_err(Error::from)?;

        let user_id = session.user_id();
        self.session = Some(session);

        Ok(user_id)
    }

    /// Загружает профиль по ID и прогоняет его через парсинг.
    ///
    /// Сеть async, HTML parsing - в blocking worker. ID числовой, так что
    /// в URL нельзя случайно подсунуть левый путь.
    pub async fn get_user(&self, user_id: u32) -> Result<User> {
        let path = format!("/users/{user_id}/");
        let html = self.fetch_html(&path).await?;

        self.parse_user_html(html, user_id).await
    }

    /// Загружает профиль текущего пользователя.
    ///
    /// Первый вызов открывает главную, достаёт `user_id` из `data-app-data`
    /// и сохраняет session-контекст. Дальше контекст уже не пересобираем.
    ///
    /// Сейчас возвращается только `User`. Балансы сюда пока не присоединяем,
    /// потому что парсинг баланса ещё не готов.
    pub async fn get_current_user(&mut self) -> Result<User> {
        let user_id = self.initialize_session().await?;
        self.get_user(user_id).await
    }
}
