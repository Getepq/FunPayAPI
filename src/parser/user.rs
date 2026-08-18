use crate::models::{Status, User};
use crate::parser::selectors::{
    AVATAR_URL_SEL, OFFLINE_STATUS_SEL, ONLINE_STATUS_SEL, REGISTRATION_DATE_SEL,
    REVIEWS_COUNT_SEL, USER_BADGE_SEL, USERNAME_SEL,
};
use crate::parser::{Error, Result};
use scraper::Html;

/// Собирает [`User`] из уже разобранного HTML-документа.
///
/// Метод не выполняет сетевых запросов. Вызывающий код запускает разбор HTML
/// через `tokio::task::spawn_blocking` и передаёт безопасный числовой
/// идентификатор пользователя.
///
/// # Errors
///
/// Возвращает ошибку, если HTML не содержит обязательного поля профиля или
/// значение обязательного поля пусто.
pub(crate) fn get_user(document: &Html, user_id: u32) -> Result<User> {
    Ok(User {
        id: user_id,
        username: parse_username(document)?,
        avatar_url: parse_avatar_url(document)?,
        status: parse_status(document)?,
        reviews_count: parse_reviews_count(document),
        registered_at: parse_registration_date(document)?,
    })
}

/// Извлекает отображаемое имя пользователя.
///
/// Имя является обязательным полем профиля. Пустая строка считается ошибкой
/// разбора.
fn parse_username(document: &Html) -> Result<String> {
    let username = document
        .select(&USERNAME_SEL)
        .next()
        .ok_or(Error::SelectorNotFound(".profile > h1 > span.mr4"))?
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    if username.is_empty() {
        return Err(Error::EmptyField("username"));
    }

    Ok(username)
}

/// Извлекает и приводит к абсолютному виду адрес изображения профиля.
///
/// Метод принимает абсолютный, относительный и протокольно-независимый адрес.
/// Отсутствие атрибута `style` или пустой адрес считаются ошибкой разбора.
fn parse_avatar_url(document: &Html) -> Result<String> {
    let avatar = document
        .select(&AVATAR_URL_SEL)
        .next()
        .ok_or(Error::SelectorNotFound(".avatar > .avatar-photo"))?;

    let style = avatar
        .value()
        .attr("style")
        .ok_or(Error::MissingAttribute("style у .avatar > .avatar-photo"))?;

    let raw_url = extract_background_url(style).ok_or(Error::EmptyField("avatar URL"))?;

    Ok(normalize_avatar_url(&raw_url))
}

/// Извлекает адрес из значения CSS-свойства `background-image`.
fn extract_background_url(style: &str) -> Option<String> {
    let start = style.find("url(")? + "url(".len();
    let end = style[start..].find(')')? + start;

    let url = style[start..end].trim().trim_matches(['\'', '"']);

    (!url.is_empty()).then(|| url.to_owned())
}

/// Приводит относительный или протокольно-независимый адрес к абсолютному виду.
fn normalize_avatar_url(url: &str) -> String {
    let url = url.trim();

    if url.starts_with("//") {
        format!("https:{url}")
    } else if url.starts_with('/') {
        format!("https://funpay.com{url}")
    } else {
        url.to_owned()
    }
}

/// Извлекает количество отзывов из текста вида `Всего <N> отзыва`.
///
/// Отсутствие блока отзывов является допустимым случаем и представляется
/// значением `None`.
fn parse_reviews_count(document: &Html) -> Option<u32> {
    document
        .select(&REVIEWS_COUNT_SEL)
        .next()?
        .text()
        .flat_map(str::split_whitespace)
        .find_map(|part| part.parse::<u32>().ok())
}

/// Извлекает дату регистрации без относительного описания давности.
///
/// Пустая дата считается ошибкой разбора.
fn parse_registration_date(document: &Html) -> Result<String> {
    let date = document
        .select(&REGISTRATION_DATE_SEL)
        .next()
        .ok_or(Error::SelectorNotFound(
            ".profile-header-cols > .param-item > h5.text-bold + div.text-nowrap",
        ))?
        .text()
        .next()
        .unwrap_or("")
        .trim()
        .to_owned();

    if date.is_empty() {
        return Err(Error::EmptyField("registration date"));
    }

    Ok(date)
}

/// Определяет состояние профиля в порядке: заблокирован, в сети, не в сети.
///
/// Блокировка проверяется первой, поскольку заблокированный профиль может не
/// содержать обычный элемент состояния пользователя.
fn parse_status(document: &Html) -> Result<Status> {
    if document.select(&USER_BADGE_SEL).any(|element| {
        element
            .value()
            .attr("class")
            .is_some_and(|class| class.split_whitespace().any(|name| name == "label-danger"))
    }) {
        return Ok(Status::Blocked);
    }

    if document.select(&ONLINE_STATUS_SEL).next().is_some() {
        return Ok(Status::Online);
    }

    if let Some(element) = document.select(&OFFLINE_STATUS_SEL).next() {
        let last_seen = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        if last_seen.is_empty() {
            return Err(Error::EmptyField("last seen"));
        }

        return Ok(Status::Offline { last_seen });
    }

    Err(Error::SelectorNotFound(
        ".profile > h1.online/offline > span.media-user-status",
    ))
}
