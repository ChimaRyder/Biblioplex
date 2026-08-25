use crate::{
    domain::{Location, OwnedCard, Tag},
    error::AppResult,
    repositories,
    storage::Database,
};

pub fn add_owned_card(db: &Database, card: &OwnedCard) -> AppResult<()> {
    repositories::create_owned(db, card)
}

pub fn add_owned_catalog_card(db: &Database, card: &OwnedCard) -> AppResult<()> {
    if !repositories::catalog_printing_exists(db, &card.printing_id)? {
        return Err(crate::error::AppError::NotFound(format!(
            "catalog printing {}",
            card.printing_id
        )));
    }
    repositories::create_owned(db, card)
}

pub fn find_owned_by_printing(db: &Database, printing_id: &str) -> AppResult<Vec<OwnedCard>> {
    repositories::find_owned_by_printing(db, printing_id)
}

pub fn import_catalog(
    db: &Database,
    cards: &[crate::domain::CatalogCard],
    version: &str,
) -> AppResult<()> {
    repositories::upsert_catalog(db, cards, version)
}

pub fn search_catalog(
    db: &Database,
    query: &str,
    limit: i64,
) -> AppResult<Vec<crate::domain::CatalogCard>> {
    repositories::search_catalog(db, query, limit)
}

pub fn list_owned(
    db: &Database,
) -> AppResult<
    Vec<(
        OwnedCard,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<crate::domain::CatalogFace>,
    )>,
> {
    repositories::list_owned(db)
}

pub fn search_owned(
    db: &Database,
    query: &str,
) -> AppResult<
    Vec<(
        OwnedCard,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<crate::domain::CatalogFace>,
    )>,
> {
    repositories::search_owned(db, query)
}

pub fn remove_owned_card(db: &Database, id: &str) -> AppResult<()> {
    repositories::delete_owned(db, id)
}
pub fn update_owned_card(db: &Database, id: &str, quantity: i64, language: &str, foil: bool, condition: &str, notes: Option<&str>) -> AppResult<()> {
    repositories::update_owned(db, id, quantity, language, foil, condition, notes)
}
pub fn create_box(db: &Database, id: &str, name: &str) -> AppResult<()> {
    repositories::create_location(
        db,
        &Location {
            id: id.into(),
            name: name.into(),
            kind: "box".into(),
            archived: false,
        },
    )
}
pub fn create_deck(db: &Database, id: &str, name: &str) -> AppResult<()> {
    repositories::create_location(
        db,
        &Location {
            id: id.into(),
            name: name.into(),
            kind: "deck".into(),
            archived: false,
        },
    )
}
pub fn move_card(
    db: &Database,
    card_id: &str,
    location_id: &str,
    quantity: i64,
    section: &str,
) -> AppResult<()> {
    repositories::assign(db, card_id, location_id, quantity, section)
}
pub fn add_tag(db: &Database, id: &str, name: &str) -> AppResult<()> {
    repositories::create_tag(
        db,
        &Tag {
            id: id.into(),
            name: name.into(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::OwnedCard, storage::Database};
    #[test]
    fn allocation_rejects_more_than_owned() {
        let db = Database::in_memory().unwrap();
        crate::repositories::upsert_catalog(
            &db,
            &[crate::domain::CatalogCard {
                uuid: "p1".into(),
                name: "Card".into(),
                set_code: "TST".into(),
                collector_number: "1".into(),
                rarity: None,
                oracle_text: None,
                mana_cost: None,
                card_type: None,
                power: None,
                toughness: None,
                scryfall_id: None,
                faces: Vec::new(),
            }],
            "test",
        )
        .unwrap();
        add_owned_card(
            &db,
            &OwnedCard {
                id: "o1".into(),
                printing_id: "p1".into(),
                quantity: 2,
                language: "en".into(),
                foil: false,
                condition: "near_mint".into(),
                notes: None,
            },
        )
        .unwrap();
        create_box(&db, "b1", "Box").unwrap();
        assert!(move_card(&db, "o1", "b1", 3, "box").is_err());
    }
}
