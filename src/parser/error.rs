use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

// todo! Описать Ошибка парсинга.
#[derive(Debug, Error)]
pub enum Error {

}