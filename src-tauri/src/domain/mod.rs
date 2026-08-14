use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedCard {
    pub id: String,
    pub printing_id: String,
    pub quantity: i64,
    pub language: String,
    pub foil: bool,
    pub condition: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogCard {
    pub uuid: String,
    pub name: String,
    pub set_code: String,
    pub collector_number: String,
    pub rarity: Option<String>,
    pub oracle_text: Option<String>,
    pub mana_cost: Option<String>,
    pub card_type: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub scryfall_id: Option<String>,
    pub faces: Vec<CatalogFace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogFace {
    pub face_order: i64,
    pub name: String,
    pub mana_cost: Option<String>,
    pub card_type: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub scryfall_id: Option<String>,
    pub cached_path: Option<String>,
    pub image_status: String,
}
