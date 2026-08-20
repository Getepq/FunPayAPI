use crate::models::{Lot, LotTypes, Offer, OfferAmount, OfferField, OfferPreview};
use crate::parser::selectors::{
    OFFER_AMOUNT_SEL, OFFER_CARD_SEL, OFFER_CHAT_SEL, OFFER_ID_INPUT_SEL, OFFER_ITEM_SEL,
    OFFER_LOT_LINK_SEL, OFFER_NAME_SEL, OFFER_PARAM_LABEL_SEL, OFFER_PARAM_SEL,
    OFFER_PARAM_VALUE_SEL, OFFER_PRICE_SEL,
};
use crate::parser::{Error, Result};
use scraper::{ElementRef, Html};

/// Собирает все превью предложений со страницы профиля `FunPay`.
///
/// Метод находит карточки предложений в профиле, определяет лот по ссылке из
/// заголовка карточки и собирает строки таблицы в [`OfferPreview`]. Сетевые
/// запросы не выполняются; вызывающий код запускает разбор через
/// `tokio::task::spawn_blocking`.
///
/// # Errors
///
/// Возвращает ошибку, если не найдено ни одной карточки предложения, если
/// обязательные поля строки пусты, либо если разметка ссылок на лот или offer
/// отличается от ожидаемого формата.
pub(crate) fn get_offer_previews(document: &Html) -> Result<Vec<OfferPreview>> {
    let previews = document
        .select(&OFFER_CARD_SEL)
        .map(parse_offer_card)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    if previews.is_empty() {
        return Err(Error::SelectorNotFound(
            ".profile-data-container > .mb20 > .offer",
        ));
    }

    tracing::debug!(offers = previews.len(), "Parsed offer previews");
    Ok(previews)
}

/// Собирает полную модель предложения из страницы деталей и соответствующего превью.
///
/// Детальная страница не повторяет все поля строки списка. Поэтому `name`,
/// `price`, `amount` и `lot` переносятся из ранее разобранного
/// [`OfferPreview`], а описание и способ передачи извлекаются из списка
/// параметров. Перед возвратом метод сверяет тип и идентификатор страницы с
/// превью, защищая от случайного объединения данных разных предложений.
///
/// # Errors
///
/// Возвращает ошибку при несовпадении страницы и превью, при пустом
/// идентификаторе или неожиданной структуре обязательных элементов HTML.
pub(crate) fn get_offer(document: &Html, preview: OfferPreview) -> Result<Offer> {
    let (actual_type, actual_id) = parse_detail_identity(document)?;
    if actual_type != preview.lot.r#type || actual_id != preview.id {
        return Err(Error::OfferMismatch {
            expected: offer_identity(&preview.lot.r#type, &preview.id),
            actual: offer_identity(&actual_type, &actual_id),
        });
    }

    let fields = parse_offer_fields(document)?;
    let amount = fields
        .iter()
        .find(|field| field.name == "Наличие")
        .map(|field| parse_amount_text(&field.value))
        .unwrap_or(preview.amount.clone());

    Ok(Offer {
        id: preview.id,
        name: preview.name,
        offer_type: field_value(&fields, |name| name.starts_with("Тип ")),
        description: field_value(&fields, |name| name == "Подробное описание"),
        price: preview.price,
        amount,
        delivery_method: field_value(&fields, |name| name.starts_with("Способ ")),
        fields,
        lot: preview.lot,
    })
}

/// Разбирает одну карточку лота и все отображаемые в ней строки предложений.
fn parse_offer_card(card: ElementRef<'_>) -> Result<Vec<OfferPreview>> {
    let lot_link = card
        .select(&OFFER_LOT_LINK_SEL)
        .next()
        .ok_or(Error::SelectorNotFound(".offer-list-title > h3 > a[href]"))?;
    let lot = parse_lot(lot_link)?;

    card.select(&OFFER_ITEM_SEL)
        .map(|item| parse_offer_preview(item, &lot))
        .collect()
}

/// Собирает превью из одной строки таблицы предложений.
fn parse_offer_preview(item: ElementRef<'_>, lot: &Lot) -> Result<OfferPreview> {
    let href = required_attribute(item, "href", "href у a.tc-item")?;
    let (offer_type, id) = parse_offer_href(&href)?;
    if offer_type != lot.r#type {
        return Err(Error::OfferMismatch {
            expected: offer_identity(&lot.r#type, "<идентификатор из строки>"),
            actual: offer_identity(&offer_type, &id),
        });
    }

    let name = required_text(
        item.select(&OFFER_NAME_SEL).next(),
        ".tc-desc > .tc-desc-text, .tc-server.hidden-xxs",
        "offer name",
    )?;
    let amount = item
        .select(&OFFER_AMOUNT_SEL)
        .next()
        .map_or_else(|| OfferAmount::Raw(String::new()), parse_amount);
    let price_element = item
        .select(&OFFER_PRICE_SEL)
        .next()
        .ok_or(Error::SelectorNotFound(".tc-item > .tc-price"))?;
    let price = required_attribute(price_element, "data-s", "data-s у .tc-item > .tc-price")?;

    Ok(OfferPreview {
        id,
        name,
        price,
        amount,
        lot: lot.clone(),
    })
}

/// Разбирает лот из ссылки и её отображаемого названия.
fn parse_lot(link: ElementRef<'_>) -> Result<Lot> {
    let href = required_attribute(link, "href", "href у ссылки на лот")?;
    let (r#type, id) = parse_lot_href(&href)?;
    let title = required_text(Some(link), ".offer-list-title > h3 > a[href]", "lot title")?;
    let (category, product) = split_lot_title(&title)?;

    Ok(Lot {
        id,
        product,
        category,
        r#type,
    })
}

/// Извлекает тип и идентификатор предложения из детальной страницы.
fn parse_detail_identity(document: &Html) -> Result<(LotTypes, String)> {
    if let Some(input) = document.select(&OFFER_ID_INPUT_SEL).next() {
        let id = required_attribute(input, "value", "value у input[name=offer_id]")?;
        return Ok((LotTypes::Common, id));
    }

    let chat = document
        .select(&OFFER_CHAT_SEL)
        .next()
        .ok_or(Error::SelectorNotFound(".chat[data-offer]"))?;
    let raw = required_attribute(chat, "data-offer", "data-offer у .chat")?;
    parse_chat_offer_identity(&raw)
}

/// Разбирает значение `data-offer` вида `lot:<id>` или `chip:<id>`.
fn parse_chat_offer_identity(value: &str) -> Result<(LotTypes, String)> {
    let (kind, id) = value.split_once(':').ok_or_else(|| Error::InvalidUrl {
        field: "data-offer",
        value: value.to_owned(),
    })?;
    let r#type = match kind {
        "lot" => LotTypes::Common,
        "chip" => LotTypes::Chips,
        _ => {
            return Err(Error::InvalidUrl {
                field: "data-offer",
                value: value.to_owned(),
            });
        }
    };
    let id = id.trim();
    if id.is_empty() {
        return Err(Error::EmptyField("offer id"));
    }
    Ok((r#type, id.to_owned()))
}

/// Разбирает ссылку на страницу предложения и определяет её тип.
fn parse_offer_href(href: &str) -> Result<(LotTypes, String)> {
    let path = url_path(href);
    let r#type = if has_path_segment(&path, "lots") && has_path_segment(&path, "offer") {
        LotTypes::Common
    } else if has_path_segment(&path, "chips") && has_path_segment(&path, "offer") {
        LotTypes::Chips
    } else {
        return Err(Error::InvalidUrl {
            field: "offer href",
            value: href.to_owned(),
        });
    };

    let query = href
        .split_once('?')
        .map(|(_, query)| query)
        .ok_or_else(|| Error::InvalidUrl {
            field: "offer href",
            value: href.to_owned(),
        })?;
    let id = query
        .split('#')
        .next()
        .unwrap_or_default()
        .split('&')
        .find_map(|parameter| parameter.split_once('='))
        .filter(|(name, _)| *name == "id")
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::InvalidUrl {
            field: "offer href",
            value: href.to_owned(),
        })?;

    Ok((r#type, id.to_owned()))
}

/// Разбирает ссылку на лот вида `/lots/<id>/` или `/chips/<id>/`.
fn parse_lot_href(href: &str) -> Result<(LotTypes, i32)> {
    let path = url_path(href);
    let segments = path.split('/').filter(|segment| !segment.is_empty());
    let mut segments = segments.peekable();

    while let Some(segment) = segments.next() {
        let r#type = match segment {
            "lots" => LotTypes::Common,
            "chips" => LotTypes::Chips,
            _ => continue,
        };
        let raw_id = segments.next().ok_or_else(|| Error::InvalidUrl {
            field: "lot href",
            value: href.to_owned(),
        })?;
        let id = raw_id.parse::<i32>().map_err(|_| Error::InvalidNumber {
            field: "lot id",
            value: raw_id.to_owned(),
        })?;
        if id <= 0 {
            return Err(Error::InvalidNumber {
                field: "lot id",
                value: raw_id.to_owned(),
            });
        }
        return Ok((r#type, id));
    }

    Err(Error::InvalidUrl {
        field: "lot href",
        value: href.to_owned(),
    })
}

/// Делит заголовок лота на категорию и продукт.
///
/// На странице профиль отображает лот как `<категория> <продукт>`. Продуктом
/// является последнее слово, поэтому категория может состоять из произвольного
/// числа слов: например, `Blox Fruits Roblox` превращается в категорию
/// `Blox Fruits` и продукт `Roblox`.
fn split_lot_title(title: &str) -> Result<(String, String)> {
    let (category, product) = title
        .rsplit_once(' ')
        .ok_or(Error::EmptyField("lot category"))?;
    let category = category.trim();
    let product = product.trim();
    if category.is_empty() {
        return Err(Error::EmptyField("lot category"));
    }
    if product.is_empty() {
        return Err(Error::EmptyField("lot product"));
    }
    Ok((category.to_owned(), product.to_owned()))
}

/// Извлекает все параметры из основного списка параметров детальной страницы.
///
/// Порядок параметров сохраняется. Пустое значение не отбрасывается: оно
/// отражает фактическую разметку и остаётся доступным через [`Offer::field`].
fn parse_offer_fields(document: &Html) -> Result<Vec<OfferField>> {
    document
        .select(&OFFER_PARAM_SEL)
        .map(|parameter| {
            let name = required_text(
                parameter.select(&OFFER_PARAM_LABEL_SEL).next(),
                ".param-item > h5",
                "offer field name",
            )?;
            let value = parameter
                .select(&OFFER_PARAM_VALUE_SEL)
                .next()
                .map(normalized_text)
                .unwrap_or_default();
            Ok(OfferField::new(name, value))
        })
        .collect()
}

/// Находит первое значение параметра, чьё название удовлетворяет предикату.
fn field_value<F>(fields: &[OfferField], matches_name: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    fields
        .iter()
        .find(|field| matches_name(&field.name))
        .map(|field| field.value.clone())
        .filter(|value| !value.is_empty())
}

/// Извлекает количество из строки таблицы, предпочитая точное значение `data-s`.
fn parse_amount(element: ElementRef<'_>) -> OfferAmount {
    let raw = element
        .value()
        .attr("data-s")
        .map_or_else(|| normalized_text(element), ToOwned::to_owned);
    parse_amount_text(&raw)
}

/// Представляет количество числом только при безопасном преобразовании всей строки.
fn parse_amount_text(raw: &str) -> OfferAmount {
    let raw = raw.trim();
    let compact = raw
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if let Ok(amount) = compact.parse::<u32>() {
        OfferAmount::Quantity(amount)
    } else {
        OfferAmount::Raw(raw.to_owned())
    }
}

/// Получает обязательный атрибут и проверяет его на пустое значение.
fn required_attribute(
    element: ElementRef<'_>,
    attribute: &'static str,
    description: &'static str,
) -> Result<String> {
    let value = element
        .value()
        .attr(attribute)
        .ok_or(Error::MissingAttribute(description))?
        .trim();
    if value.is_empty() {
        return Err(Error::EmptyField(description));
    }
    Ok(value.to_owned())
}

/// Получает обязательный текст из необязательного элемента.
fn required_text(
    element: Option<ElementRef<'_>>,
    selector: &'static str,
    field: &'static str,
) -> Result<String> {
    let text = normalized_text(element.ok_or(Error::SelectorNotFound(selector))?);
    if text.is_empty() {
        return Err(Error::EmptyField(field));
    }
    Ok(text)
}

/// Нормализует пробелы в тексте HTML-элемента.
fn normalized_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Возвращает путь абсолютной или относительной ссылки без query-строки.
fn url_path(href: &str) -> String {
    let without_fragment = href.split('#').next().unwrap_or_default();
    let without_query = without_fragment.split('?').next().unwrap_or_default();
    if let Some(scheme_end) = without_query.find("://") {
        let authority = &without_query[scheme_end + 3..];
        let path_start = authority.find('/').unwrap_or(authority.len());
        return authority[path_start..].to_owned();
    }
    without_query.to_owned()
}

/// Проверяет, что путь содержит отдельный сегмент.
fn has_path_segment(path: &str, expected: &str) -> bool {
    path.split('/').any(|segment| segment == expected)
}

/// Формирует понятное представление идентичности предложения для сообщения ошибки.
fn offer_identity(r#type: &LotTypes, id: &str) -> String {
    let kind = match r#type {
        LotTypes::Common => "lot",
        LotTypes::Chips => "chip",
    };
    format!("{kind}:{id}")
}

// Тесты
#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE_HTML: &str = r#"
        <div class="profile-data-container"><div class="mb20">
            <div class="offer">
                <div class="offer-list-title"><h3><a href="/lots/1155/">Blox Fruits Roblox</a></h3></div>
                <a class="tc-item" href="/lots/offer?id=700">
                    <div class="tc-desc"><div class="tc-desc-text">Fiend yeti, Предметы, Почта</div></div>
                    <div class="tc-amount">998</div>
                    <div class="tc-price" data-s="889.205703"><div>889.21 ₽</div></div>
                </a>
                <a class="tc-item" href="/lots/offer?id=701">
                    <div class="tc-desc"><div class="tc-desc-text">Gamepass</div></div>
                    <div class="tc-price" data-s="10"><div>10 ₽</div></div>
                </a>
            </div>
            <div class="offer">
                <div class="offer-list-title"><h3><a href="/chips/99/">Робуксы Roblox</a></h3></div>
                <a class="tc-item" href="/chips/offer?id=1-2-99-3-0">
                    <div class="tc-server hidden-xxs">Roblox Plus</div>
                    <div class="tc-amount" data-s="26253">26 253</div>
                    <div class="tc-price" data-s="0"><div>0.97 ₽</div></div>
                </a>
            </div>
        </div></div>
    "#;

    const COMMON_HTML: &str = r#"
        <form action="/orders/new"><input name="offer_id" value="700"></form>
        <div class="page-content"><div class="param-list">
            <div class="row"><div><div class="param-item"><h5>Тип подписки</h5><div>Pro</div></div></div></div>
            <div class="param-item"><h5>Способ получения</h5><div>С заходом на аккаунт</div></div>
            <div class="param-item"><h5>Краткое описание</h5><div>Claude Pro</div></div>
            <div class="param-item"><h5>Подробное описание</h5><div>Автоматическая активация</div></div>
        </div></div>
    "#;

    const CHIPS_HTML: &str = r#"
        <div class="chat" data-offer="chip:1-2-99-3-0"></div>
        <div class="page-content"><div class="param-list">
            <div class="row"><div><div class="param-item"><h5>Способ передачи</h5><div>Roblox Plus</div></div></div></div>
            <div class="param-item"><h5>Наличие</h5><div>26 140 ед. робуксов</div></div>
        </div></div>
    "#;

    fn profile_previews() -> Vec<OfferPreview> {
        let document = Html::parse_document(PROFILE_HTML);
        get_offer_previews(&document).expect("Превью из профиля должны быть разобраны")
    }

    fn preview_by_id(id: &str) -> OfferPreview {
        profile_previews()
            .into_iter()
            .find(|preview| preview.id == id)
            .unwrap_or_else(|| panic!("В фикстуре отсутствует превью {id}"))
    }

    #[test]
    fn parses_common_and_chips_previews() {
        let previews = profile_previews();
        assert_eq!(previews.len(), 3);

        let common = &previews[0];
        assert_eq!(common.id, "700");
        assert_eq!(common.amount, OfferAmount::Quantity(998));
        assert_eq!(common.lot.category, "Blox Fruits");
        assert_eq!(common.lot.product, "Roblox");
        assert_eq!(common.lot.r#type, LotTypes::Common);

        assert_eq!(previews[1].amount, OfferAmount::Raw(String::new()));

        let chips = &previews[2];
        assert_eq!(chips.id, "1-2-99-3-0");
        assert_eq!(chips.amount, OfferAmount::Quantity(26_253));
        assert_eq!(chips.lot.id, 99);
        assert_eq!(chips.lot.r#type, LotTypes::Chips);
    }

    #[test]
    fn parses_common_offer_fields_from_detail_page() {
        let offer = get_offer(&Html::parse_document(COMMON_HTML), preview_by_id("700"))
            .expect("Обычный offer должен быть разобран");

        assert_eq!(offer.offer_type.as_deref(), Some("Pro"));
        assert_eq!(
            offer.delivery_method.as_deref(),
            Some("С заходом на аккаунт")
        );
        assert_eq!(
            offer.description.as_deref(),
            Some("Автоматическая активация")
        );
        assert_eq!(offer.amount, OfferAmount::Quantity(998));
        assert_eq!(offer.field("краткое   описание"), Some("Claude Pro"));
        assert_eq!(offer.fields.len(), 4);
    }

    #[test]
    fn parses_chips_amount_and_rejects_mismatched_preview() {
        let offer = get_offer(
            &Html::parse_document(CHIPS_HTML),
            preview_by_id("1-2-99-3-0"),
        )
        .expect("Chips offer должен быть разобран");
        assert_eq!(
            offer.amount,
            OfferAmount::Raw("26 140 ед. робуксов".to_owned())
        );
        assert_eq!(offer.delivery_method.as_deref(), Some("Roblox Plus"));
        assert_eq!(offer.field("наличие"), Some("26 140 ед. робуксов"));

        let error = get_offer(&Html::parse_document(CHIPS_HTML), preview_by_id("700"))
            .expect_err("Парсер не должен объединять разные offer-ы");
        assert!(matches!(error, Error::OfferMismatch { .. }));
    }

    #[test]
    fn splits_lot_title_and_rejects_invalid_offer_link() {
        assert_eq!(
            split_lot_title("Blox Fruits Roblox").expect("Заголовок лота должен быть разобран"),
            ("Blox Fruits".to_owned(), "Roblox".to_owned())
        );
        assert!(matches!(
            parse_offer_href("https://funpay.com/lots/1155/"),
            Err(Error::InvalidUrl {
                field: "offer href",
                ..
            })
        ));
    }
}
