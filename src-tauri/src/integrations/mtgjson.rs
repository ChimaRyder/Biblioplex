use crate::{
    domain::{CatalogCard, CatalogFace},
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
    #[serde(default)]
    colors: Vec<String>,
    #[serde(rename = "type")]
    card_type: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    identifiers: Option<Identifiers>,
    #[serde(default)]
    faces: Vec<Face>,
    #[serde(rename = "otherFaceIds", default)]
    other_face_ids: Vec<String>,
    #[serde(rename = "faceName")]
    face_name: Option<String>,
    side: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Face {
    name: String,
    #[serde(rename = "manaCost")] mana_cost: Option<String>,
    #[serde(rename = "type")] card_type: Option<String>,
    text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
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
        let cards_by_uuid: std::collections::HashMap<String, Card> = set.cards.into_iter().map(|card| (card.uuid.clone(), card)).collect();
        for card in cards_by_uuid.values() {
            if card.other_face_ids.iter().any(|id| id < &card.uuid) { continue; }
            let mut linked: Vec<&Card> = std::iter::once(card).chain(card.other_face_ids.iter().filter_map(|id| cards_by_uuid.get(id))).collect();
            linked.sort_by_key(|face| match face.side.as_deref() { Some("a") => 0, Some("b") => 1, _ => 2 });
            let parsed_faces = if card.faces.is_empty() {
                if linked.len() > 1 {
                    linked.iter().enumerate().map(|(index, face)| CatalogFace { face_order: index as i64, name: face.face_name.clone().unwrap_or_else(|| face.name.split(" // ").nth(index).unwrap_or(&face.name).trim().into()), mana_cost: face.mana_cost.clone(), card_type: face.card_type.clone(), oracle_text: face.text.clone(), power: face.power.clone(), toughness: face.toughness.clone(), scryfall_id: face.identifiers.as_ref().and_then(|i| i.scryfall_id.clone()).or_else(|| card.identifiers.as_ref().and_then(|i| i.scryfall_id.clone())), cached_path: None, image_status: "missing".into() }).collect()
                } else if let Some(face_name) = &card.face_name {
                    vec![CatalogFace { face_order: 0, name: face_name.clone(), mana_cost: card.mana_cost.clone(), card_type: card.card_type.clone(), oracle_text: card.text.clone(), power: card.power.clone(), toughness: card.toughness.clone(), scryfall_id: card.identifiers.as_ref().and_then(|i| i.scryfall_id.clone()), cached_path: None, image_status: "missing".into() }]
                } else { Vec::new() }
                } else {
                card.faces.iter().enumerate().map(|(index, face)| CatalogFace { face_order: index as i64, name: face.name.clone(), mana_cost: face.mana_cost.clone(), card_type: face.card_type.clone(), oracle_text: face.text.clone(), power: face.power.clone(), toughness: face.toughness.clone(), scryfall_id: face.identifiers.as_ref().and_then(|i| i.scryfall_id.clone()), cached_path: None, image_status: "missing".into() }).collect()
            };
            result.push(CatalogCard {
                uuid: card.uuid.clone(),
                name: card.name.clone(),
                set_code: set_code.clone(),
                collector_number: card.collector_number.clone(),
                rarity: card.rarity.clone(),
                oracle_text: card.text.clone(),
                mana_cost: card.mana_cost.clone(),
                colors: card.colors.clone(),
                card_type: card.card_type.clone(),
                power: card.power.clone(),
                toughness: card.toughness.clone(),
                scryfall_id: card.identifiers.as_ref().and_then(|i| i.scryfall_id.clone()),
                faces: parsed_faces,
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
