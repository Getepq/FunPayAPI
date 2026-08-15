/// Создаёт лениво инициализируемый статический CSS-селектор.
///
/// CSS-селектор парсится один раз при первом обращении к статике.
/// Некорректный литерал означает ошибку разработчика и приводит к панике.
macro_rules! selector {
    ($name:ident, $css:literal $(,)?) => {
        static $name: ::std::sync::LazyLock<::scraper::Selector> =
            ::std::sync::LazyLock::new(|| {
                ::scraper::Selector::parse($css)
                    .expect(concat!("некорректный внутренний CSS-селектор: ", $css))
            });
    };
}


mod chat;
mod order;
mod offer;
mod user;