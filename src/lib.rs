//! Неофициальная асинхронная библиотека для работы с `FunPay`.
//!
//! Библиотека отправляет HTTP-запросы асинхронно, а разбор HTML выполняет
//! в отдельной задаче с блокирующей операцией через `tokio::task::spawn_blocking`.
//!
//! Разметка `FunPay` может измениться. При обновлении сайта следует проверить
//! CSS-селекторы, используемые внутренними модулями разбора.

mod client;
mod error;
mod network;
mod parser;

pub use client::Client;
pub mod models;
pub use error::{Error, Result};
