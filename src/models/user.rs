use serde::{Deserialize, Serialize};
use crate::models::Balances;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub avatar_url: String,
    pub status: Status,
    pub reviews_count: Option<u32>,
    pub registered_at: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    pub user: User,
    pub balance: Balances
}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Online,
    Offline {
        last_seen: String,
    },
    Blocked,
}