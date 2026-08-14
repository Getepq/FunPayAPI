use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub avatar: String,
    pub active: bool,
    pub balance: Option<Balance>,
    pub reviews: u32,
    pub created_at: String,
    pub csrf_token: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    pub rub: String,
    pub usd: String,
    pub eur: String,
}
