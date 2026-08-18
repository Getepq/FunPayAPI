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

// Тесты
#[cfg(test)]
mod tests {
    use super::*;

    const BALANCES_HTML: &str = r#"
        <h1 class="page-header balances-header page-header-no-hr">
            Финансы<span class="balances-list"><span class="balances-delimiter">·</span><span class="balances-value">4.97 ₽</span><span class="balances-delimiter">·</span><span class="balances-value">0 $</span><span class="balances-delimiter">·</span><span class="balances-value">0 €</span></span>
        </h1>
    "#;

    #[test]
    fn parses_balances_from_real_funpay_markup() {
        let document = Html::parse_fragment(BALANCES_HTML);

        let balances = get_balances(&document).expect("Балансы должны быть разобраны");

        assert_eq!(balances.get(Currency::Rub), Some("4.97"));
        assert_eq!(balances.get(Currency::Usd), Some("0"));
        assert_eq!(balances.get(Currency::Eur), Some("0"));
    }

    #[test]
    fn ignores_balance_delimiters() {
        let document = Html::parse_fragment(BALANCES_HTML);
        let values_count = document.select(&BALANCE_VALUE_SEL).count();

        assert_eq!(values_count, 3);
    }

    #[test]
    fn returns_error_when_balance_elements_are_missing() {
        let document = Html::parse_fragment("<main></main>");

        assert!(matches!(
            get_balances(&document),
            Err(Error::SelectorNotFound(_))
        ));
    }

    #[test]
    fn returns_error_for_unknown_currency() {
        assert!(matches!(
            parse_balance("10 ¥"),
            Err(Error::EmptyField("currency in balance"))
        ));
    }

    #[test]
    fn returns_error_for_empty_amount() {
        assert!(matches!(
            parse_balance("₽"),
            Err(Error::EmptyField("balance amount"))
        ));
    }
}
