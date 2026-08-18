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

// Тесты
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_online_profile_with_reviews_from_real_markup() {
        let document = Html::parse_document(
            r#"
            <div class="profile">
                <h1 class="online">
                    <span class="mr4">Wenix1</span>
                    <span class="media-user-status">Онлайн</span>
                </h1>
            </div>
            <div class="avatar">
                <div class="avatar-photo" style="background-image: url(https://sfunpay.com/s/avatar/hj/td/hjtdeogfcq1brclbldai.jpg);"></div>
            </div>
            <div class="profile-header-col-rating">
                <div class="rating-full-count"><a>Всего 3 отзыва</a></div>
            </div>
            <div class="profile-header-cols">
                <div class="param-item">
                    <h5 class="text-bold">На сайте с</h5>
                    <div class="text-nowrap">24 марта 2024, 18:55 <span>2 года назад</span></div>
                </div>
            </div>
            "#,
        );

        let user = get_user(&document, 10_486_765).expect("Профиль должен быть разобран");

        assert_eq!(user.id, 10_486_765);
        assert_eq!(user.username, "Wenix1");
        assert_eq!(
            user.avatar_url,
            "https://sfunpay.com/s/avatar/hj/td/hjtdeogfcq1brclbldai.jpg"
        );
        assert_eq!(user.status, Status::Online);
        assert_eq!(user.reviews_count, Some(3));
        assert_eq!(user.registered_at, "24 марта 2024, 18:55");
    }

    #[test]
    fn parses_offline_profile_from_real_markup() {
        let document = Html::parse_document(
            r#"
            <div class="profile">
                <h1 class="offline">
                    <span class="mr4">casaaaaaaaaa</span>
                    <span class="media-user-status">Был сегодня в 11:02 <span>(35 минут назад)</span></span>
                </h1>
            </div>
            <div class="avatar">
                <div class="avatar-photo" style="background-image: url(https://sfunpay.com/s/avatar/f5/ro/f5ro8nys68vgf2bvynbb.jpg);"></div>
            </div>
            <div class="profile-header-col-rating">
                <div class="rating-full-count"><a>Всего 183 отзыва</a></div>
            </div>
            <div class="profile-header-cols">
                <div class="param-item">
                    <h5 class="text-bold">На сайте с</h5>
                    <div class="text-nowrap">14 июня 2024, 16:32 <span>2 года назад</span></div>
                </div>
            </div>
            "#,
        );

        let user = get_user(&document, 123).expect("Профиль должен быть разобран");

        assert_eq!(user.username, "casaaaaaaaaa");
        assert_eq!(user.reviews_count, Some(183));
        assert_eq!(user.registered_at, "14 июня 2024, 16:32");
        assert_eq!(
            user.status,
            Status::Offline {
                last_seen: "Был сегодня в 11:02 (35 минут назад)".to_owned(),
            }
        );
    }

    #[test]
    fn prioritizes_blocked_profile_from_real_markup() {
        let document = Html::parse_document(
            r#"
            <div class="profile">
                <h1>
                    <span class="mr4">ximivo556</span>
                    <small class="user-badges"><span class="label label-danger">заблокирован</span></small>
                </h1>
            </div>
            <div class="avatar">
                <div class="avatar-photo" style="background-image: url(https://sfunpay.com/s/avatar/dp/u0/dpu0z59apiaa4be9y9p5.jpg);"></div>
            </div>
            <div class="profile-header-cols">
                <div class="param-item">
                    <h5 class="text-bold">На сайте с</h5>
                    <div class="text-nowrap">28 августа 2021, 19:19 <span>5 лет назад</span></div>
                </div>
            </div>
            "#,
        );

        let user = get_user(&document, 456).expect("Профиль должен быть разобран");

        assert_eq!(user.username, "ximivo556");
        assert_eq!(user.reviews_count, None);
        assert_eq!(user.registered_at, "28 августа 2021, 19:19");
        assert_eq!(user.status, Status::Blocked);
    }

    #[test]
    fn parses_relative_avatar_and_ignores_non_danger_badge() {
        let document = Html::parse_document(
            r#"
            <div class="profile">
                <h1 class="online">
                    <span class="mr4">FunPay</span>
                    <small class="user-badges"><span class="label label-success">поддержка</span></small>
                    <span class="media-user-status">Онлайн</span>
                </h1>
            </div>
            <div class="avatar">
                <div class="avatar-photo" style="background-image: url(/img/layout/avatar.png);"></div>
            </div>
            <div class="profile-header-cols">
                <div class="param-item">
                    <h5 class="text-bold">На сайте с</h5>
                    <div class="text-nowrap">24 августа 2015, 23:39 <span>11 лет назад</span></div>
                </div>
            </div>
            "#,
        );

        let user = get_user(&document, 1).expect("Профиль должен быть разобран");

        assert_eq!(user.avatar_url, "https://funpay.com/img/layout/avatar.png");
        assert_eq!(user.status, Status::Online);
        assert_eq!(user.reviews_count, None);
    }

    #[test]
    fn returns_error_when_username_is_missing() {
        let document = Html::parse_fragment(
            r#"
            <div class="profile">
                <h1 class="online"><span class="media-user-status">Онлайн</span></h1>
            </div>
            "#,
        );

        assert!(matches!(
            get_user(&document, 1),
            Err(Error::SelectorNotFound(_))
        ));
    }
}
