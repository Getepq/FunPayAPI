//! Неофициальная async-библиотека для `FunPay`.
//!
//! Тут всё довольно просто: HTTP-запросы отправляются асинхронно, а HTML парсится
//! в `spawn_blocking`, чтобы не стопить рантайм токио.
//!
//! Разметка `FunPay` может поменяться в любой момент, так что селекторы
//! лучше периодически перепроверять на свежем HTML.

mod client;
mod error;
mod models;
mod network;
mod parser;

pub use client::Client;
pub use error::{Error, Result};
pub use models::{
    Balances, Chat, ChatPreview, CurrentUser, Lot, LotTypes, Message, MsgFrom, MsgTypes, Offer,
    OfferPreview, Order, OrderPreview, Status, User,
};
