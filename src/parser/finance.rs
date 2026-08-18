use crate::models::{Balances, Currency};
use crate::parser::selectors::BALANCE_VALUE_SEL;
use crate::parser::{Error, Result};
use scraper::Html;

/// Собирает балансы текущего пользователя из уже разобранного HTML-документа.
///
/// Метод находит элементы `span.balances-value` внутри `span.balances-list`,
/// определяет валюту по последнему символу и создаёт [`Balances`]. Сетевые
/// запросы не выполняются; вызывающий код запускает разбор HTML через
/// `tokio::task::spawn_blocking`.
///
/// # Errors
///
/// Возвращает [`Error::SelectorNotFound`], если страница `FunPay` не содержит
/// ни одного элемента баланса. Возвращает [`Error::EmptyField`], если сумма
/// отсутствует либо строка не заканчивается известным символом валюты.
pub(crate) fn get_balances(document: &Html) -> Result<Balances> {
    let balances = document
        .select(&BALANCE_VALUE_SEL)
        .map(|element| {
            let raw = element.text().collect::<String>();
            parse_balance(&raw)
        })
        .collect::<Result<Balances>>()?;

    if balances.iter().next().is_none() {
        return Err(Error::SelectorNotFound(
            "span.balances-list > span.balances-value",
        ));
    }

    tracing::debug!(?balances, "Parsed balances");
    Ok(balances)
}

/// Разделяет текст одного баланса на валюту и числовую часть суммы.
///
/// `FunPay` помещает символ валюты в конец строки: например, `"1 ₽"`,
/// `"2 €"` или `"3 $"`. Метод удаляет символ только в конце строки, поэтому
/// такой же символ внутри суммы не влияет на определение [`Currency`].
///
/// Возвращает ошибку, если строка не содержит известного символа валюты или
/// числовая часть суммы пуста.
fn parse_balance(raw: &str) -> Result<(Currency, String)> {
    let raw = raw.trim();

    let (amount, currency) = if let Some(amount) = raw.strip_suffix('₽') {
        (amount, Currency::Rub)
    } else if let Some(amount) = raw.strip_suffix('$') {
        (amount, Currency::Usd)
    } else if let Some(amount) = raw.strip_suffix('€') {
        (amount, Currency::Eur)
    } else {
        return Err(Error::EmptyField("currency in balance"));
    };

    let amount = amount.trim();
    if amount.is_empty() {
        return Err(Error::EmptyField("balance amount"));
    }

    Ok((currency, amount.to_owned()))
}
