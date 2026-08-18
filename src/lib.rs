//! Неофициальная асинхронная библиотека для работы с `FunPay`.
//!
//! Библиотека отправляет HTTP-запросы асинхронно, а разбор HTML выполняет
//! в отдельной задаче с блокирующей операцией через `tokio::task::spawn_blocking`.
//!
//! Разметка `FunPay` может измениться. При обновлении сайта следует проверить
//! CSS-селекторы, используемые внутренними модулями разбора.

mod client;
mod error;
mod models;
mod network;
mod parser;

pub use client::Client;
pub use error::{Error, Result};
pub use models::{
    Balances, Chat, ChatPreview, Currency, CurrentUser, Lot, LotTypes, Message, MsgFrom, MsgTypes,
    Offer, OfferPreview, Order, OrderPreview, Status, User,
};
