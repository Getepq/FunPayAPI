use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Валюта, поддерживаемая страницей баланса `FunPay`.
///
/// При сериализации варианты записываются в формате `snake_case`: например,
/// [`Currency::Rub`] становится строкой `"rub"`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Currency {
    /// Российский рубль.
    Rub,
    /// Евро.
    Eur,
    /// Доллар США.
    Usd,
}

/// Балансы пользователя по валютам, полученные со страницы `FunPay`.
///
/// Ключ определяет валюту, а значение хранит числовую часть баланса без
/// символа валюты. Например, `"4.97"` соответствует [`Currency::Rub`].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Balances {
    /// Числовые значения балансов, сгруппированные по валютам.
    pub values: HashMap<Currency, String>,
}

impl Balances {
    /// Возвращает числовую часть баланса для указанной валюты.
    ///
    /// Возвращает `None`, если страница `FunPay` не содержит баланс в указанной
    /// валюте.
    pub fn get(&self, currency: Currency) -> Option<&str> {
        self.values.get(&currency).map(String::as_str)
    }

    /// Последовательно возвращает пары валют и соответствующих сумм.
    ///
    /// Порядок элементов не определён, поскольку значения хранятся в
    /// [`HashMap`]. Для фиксированного порядка отображения его задаёт
    /// вызывающий код.
    pub fn iter(&self) -> impl Iterator<Item = (Currency, &str)> {
        self.values
            .iter()
            .map(|(&currency, amount)| (currency, amount.as_str()))
    }
}

/// Собирает [`Balances`] из пар валюты и числовой части суммы.
///
/// Если входная последовательность содержит одну валюту несколько раз,
/// сохраняется последнее значение.
impl FromIterator<(Currency, String)> for Balances {
    fn from_iter<T: IntoIterator<Item = (Currency, String)>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}
