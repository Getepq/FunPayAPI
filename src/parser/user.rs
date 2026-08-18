use crate::models::{Status, User};
use crate::parser::selectors::{
    AVATAR_URL_SEL, OFFLINE_STATUS_SEL, ONLINE_STATUS_SEL, REGISTRATION_DATE_SEL,
    REVIEWS_COUNT_SEL, USER_BADGE_SEL, USERNAME_SEL,
};
use crate::parser::{Error, Result};
use scraper::Html;

/// Синхронно собирает `User` из уже загруженного HTML-документа.
///
/// Функция не делает запрсов и не должна запускаться прямо в async-таске.
/// Вызывающий слой кладёт её в `tokio::task::spawn_blocking`, а сюда передаёт
/// только `Html` и безопасный числовой `user_id`.
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

/// Парсит username. Поле обязательное: без него профиль считаем битым.
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

/// Парсит avatar URL.
///
/// Аватарка опциональна: `FunPay` может отдать стандартный `/img/...` path,
/// абсолютный CDN URL или вообще не отдать style. В последнем случае
/// возвращаем `None`, а не прячем проблему за пустой строкой.
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

/// Вытаскивает URL из inline CSS вида `background-image: url(...)`.
fn extract_background_url(style: &str) -> Option<String> {
    let start = style.find("url(")? + "url(".len();
    let end = style[start..].find(')')? + start;

    let url = style[start..end].trim().trim_matches(['\'', '"']);

    (!url.is_empty()).then(|| url.to_owned())
}

/// Нормализует relative и protocol-relative URL в абсолютный URL.
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

/// Достаёт число отзывов из текста `Всего <N> отзыва`.
///
/// Отсутствие блока отзывов - валидный кейс, поэтому возвращаем `None`.
fn parse_reviews_count(document: &Html) -> Option<u32> {
    document
        .select(&REVIEWS_COUNT_SEL)
        .next()?
        .text()
        .flat_map(str::split_whitespace)
        .find_map(|part| part.parse::<u32>().ok())
}

/// Парсит только абсолютную дату регистрации, без подсказки `N лет назад`.
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

/// Определяет статус в порядке приоритета: blocked, online, offline.
///
/// Заблокированный профиль может не иметь обычного `media-user-status`,
/// поэтому danger-бейдж проверяем первым.
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
