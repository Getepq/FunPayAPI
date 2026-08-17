use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balances {
    pub rub: String,
    pub usd: String,
    pub eur: String,
}
