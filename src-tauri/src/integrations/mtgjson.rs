use crate::{
    domain::CatalogCard,
    error::{AppError, AppResult},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Payload {
    data: std::collections::HashMap<String, Set>,
}
#[derive(Debug, Deserialize)]
struct Set {
    #[serde(default)]
    cards: Vec<Card>,
}
#[derive(Debug, Deserialize)]
struct Card {
    uuid: String,
    name: String,
    #[serde(rename = "number")]
    collector_number: String,
    rarity: Option<String>,
    text: Option<String>,
    #[serde(rename = "manaCost")]
    mana_cost: Option<String>,
    #[serde(rename = "type")]
    card_type: Option<String>,
    identifiers: Option<Identifiers>,
}
#[derive(Debug, Deserialize)]
struct Identifiers {
    #[serde(rename = "scryfallId")]
    scryfall_id: Option<String>,
}

pub fn parse_all_printings(input: &str) -> AppResult<Vec<CatalogCard>> {
    let payload: Payload =
        serde_json::from_str(input).map_err(|e| AppError::Import(e.to_string()))?;
    let mut result = Vec::new();
    for (set_code, set) in payload.data {
        for card in set.cards {
            result.push(CatalogCard {
                uuid: card.uuid,
                name: card.name,
                set_code: set_code.clone(),
                collector_number: card.collector_number,
                rarity: card.rarity,
                oracle_text: card.text,
                mana_cost: card.mana_cost,
                card_type: card.card_type,
                scryfall_id: card.identifiers.and_then(|i| i.scryfall_id),
            });
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::parse_all_printings;
    #[test]
    fn projects_required_mtgjson_fields() {
        let input = r#"{"data":{"TST":{"cards":[{"uuid":"1","name":"Example","number":"1","rarity":"common","text":"Draw a card.","identifiers":{"scryfallId":"sf1"}}]}}}"#;
        let cards = parse_all_printings(input).unwrap();
        assert_eq!(cards[0].name, "Example");
        assert_eq!(cards[0].scryfall_id.as_deref(), Some("sf1"));
    }
}
