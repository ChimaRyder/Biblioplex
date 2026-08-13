use crate::{
    domain::{Location, OwnedCard},
    error::{AppError, AppResult},
    storage::Database,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Backup {
    pub format_version: u32,
    pub owned_cards: Vec<OwnedCard>,
    pub locations: Vec<Location>,
}

pub fn export(db: &Database) -> AppResult<String> {
    let mut cards = Vec::new();
    let mut stmt = db
        .connection
        .prepare("SELECT id,printing_id,quantity,language,foil,condition,notes FROM owned_cards")?;
    for row in stmt.query_map([], |r| {
        Ok(OwnedCard {
            id: r.get(0)?,
            printing_id: r.get(1)?,
            quantity: r.get(2)?,
            language: r.get(3)?,
            foil: r.get::<_, i64>(4)? != 0,
            condition: r.get(5)?,
            notes: r.get(6)?,
        })
    })? {
        cards.push(row?);
    }
    let mut locations = Vec::new();
    let mut stmt = db
        .connection
        .prepare("SELECT id,name,kind,archived FROM locations")?;
    for row in stmt.query_map([], |r| {
        Ok(Location {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            archived: r.get::<_, i64>(3)? != 0,
        })
    })? {
        locations.push(row?);
    }
    serde_json::to_string(&Backup {
        format_version: 1,
        owned_cards: cards,
        locations,
    })
    .map_err(|e| AppError::Import(e.to_string()))
}

pub fn import(db: &Database, input: &str) -> AppResult<()> {
    let backup: Backup =
        serde_json::from_str(input).map_err(|e| AppError::Import(e.to_string()))?;
    if backup.format_version != 1 {
        return Err(AppError::Import("unsupported format version".into()));
    }
    let tx = db.connection.unchecked_transaction()?;
    for location in backup.locations {
        tx.execute("INSERT INTO locations(id,name,kind,archived) VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,kind=excluded.kind,archived=excluded.archived", params![location.id, location.name, location.kind, location.archived as i64])?;
    }
    for card in backup.owned_cards {
        if card.quantity <= 0 {
            return Err(AppError::Validation("quantity must be positive".into()));
        }
        tx.execute("INSERT INTO owned_cards(id,printing_id,quantity,language,foil,condition,notes) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET printing_id=excluded.printing_id,quantity=excluded.quantity,language=excluded.language,foil=excluded.foil,condition=excluded.condition,notes=excluded.notes", params![card.id, card.printing_id, card.quantity, card.language, card.foil as i64, card.condition, card.notes])?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::OwnedCard, repositories, services};
    #[test]
    fn backup_round_trip_preserves_owned_cards() {
        let db = Database::in_memory().unwrap();
        repositories::upsert_catalog(
            &db,
            &[crate::domain::CatalogCard {
                uuid: "p".into(),
                name: "Card".into(),
                set_code: "T".into(),
                collector_number: "1".into(),
                rarity: None,
                oracle_text: None,
                mana_cost: None,
                card_type: None,
                scryfall_id: None,
            }],
            "v",
        )
        .unwrap();
        services::add_owned_card(
            &db,
            &OwnedCard {
                id: "o".into(),
                printing_id: "p".into(),
                quantity: 1,
                language: "en".into(),
                foil: false,
                condition: "near_mint".into(),
                notes: Some("test".into()),
            },
        )
        .unwrap();
        let data = export(&db).unwrap();
        let restored = Database::in_memory().unwrap();
        repositories::upsert_catalog(
            &restored,
            &[crate::domain::CatalogCard {
                uuid: "p".into(),
                name: "Card".into(),
                set_code: "T".into(),
                collector_number: "1".into(),
                rarity: None,
                oracle_text: None,
                mana_cost: None,
                card_type: None,
                scryfall_id: None,
            }],
            "v",
        )
        .unwrap();
        import(&restored, &data).unwrap();
        let count: i64 = restored
            .connection
            .query_row("SELECT count(*) FROM owned_cards", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
