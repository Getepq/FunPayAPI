use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferPreview {
    pub id: String,
    pub name: String,
    pub price: String,
    pub amount: u32,
    pub lot: Lot,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub price: String,
    pub amount: u32,
    pub method: Option<String>,
    pub lot: Lot,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lot {
    pub id: i32,
    pub product: String,
    pub category: String,
    pub r#type: LotTypes,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LotTypes {
    Common,
    Currency,
}
