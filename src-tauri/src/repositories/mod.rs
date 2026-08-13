use crate::storage::Database;
use crate::{
    domain::{CatalogCard, Location, OwnedCard, Tag},
    error::{AppError, AppResult},
};
use rusqlite::{params, OptionalExtension};

pub fn upsert_catalog(db: &Database, cards: &[CatalogCard], version: &str) -> AppResult<()> {
    let tx = db.connection.unchecked_transaction()?;
    for card in cards {
        tx.execute("INSERT INTO printings(id,name,set_code,collector_number,rarity,oracle_text,mana_cost,card_type,scryfall_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET name=excluded.name,set_code=excluded.set_code,collector_number=excluded.collector_number,rarity=excluded.rarity,oracle_text=excluded.oracle_text,mana_cost=excluded.mana_cost,card_type=excluded.card_type,scryfall_id=excluded.scryfall_id",
            params![card.uuid, card.name, card.set_code, card.collector_number, card.rarity, card.oracle_text, card.mana_cost, card.card_type, card.scryfall_id])?;
    }
    tx.execute("INSERT INTO catalog_metadata(id,version,imported_at) VALUES (1,?1,datetime('now')) ON CONFLICT(id) DO UPDATE SET version=excluded.version, imported_at=excluded.imported_at", [version])?;
    tx.commit()?;
    Ok(())
}

pub fn search_catalog(db: &Database, query: &str, limit: i64) -> AppResult<Vec<CatalogCard>> {
    let limit = limit.clamp(1, 100);
    let pattern = format!("%{}%", query.trim());
    let mut statement = db.connection.prepare(
        "SELECT id,name,set_code,collector_number,rarity,oracle_text,mana_cost,card_type,scryfall_id
         FROM printings
         WHERE name LIKE ?1 COLLATE NOCASE
            OR set_code LIKE ?1 COLLATE NOCASE
            OR collector_number LIKE ?1 COLLATE NOCASE
         ORDER BY name COLLATE NOCASE, set_code COLLATE NOCASE, collector_number
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![pattern, limit], |row| {
        Ok(CatalogCard {
            uuid: row.get(0)?,
            name: row.get(1)?,
            set_code: row.get(2)?,
            collector_number: row.get(3)?,
            rarity: row.get(4)?,
            oracle_text: row.get(5)?,
            mana_cost: row.get(6)?,
            card_type: row.get(7)?,
            scryfall_id: row.get(8)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_owned(db: &Database, card: &OwnedCard) -> AppResult<()> {
    if card.quantity <= 0 {
        return Err(AppError::Validation("quantity must be positive".into()));
    }
    db.connection.execute(
        "INSERT INTO owned_cards VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            card.id,
            card.printing_id,
            card.quantity,
            card.language,
            card.foil as i64,
            card.condition,
            card.notes
        ],
    )?;
    Ok(())
}

pub fn catalog_printing_exists(db: &Database, printing_id: &str) -> AppResult<bool> {
    Ok(db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM printings WHERE id=?1)",
        [printing_id],
        |row| row.get(0),
    )?)
}

pub fn find_catalog_card(db: &Database, printing_id: &str) -> AppResult<Option<CatalogCard>> {
    db.connection
        .query_row(
            "SELECT id,name,set_code,collector_number,rarity,oracle_text,mana_cost,card_type,scryfall_id
             FROM printings WHERE id=?1",
            [printing_id],
            |row| {
                Ok(CatalogCard {
                    uuid: row.get(0)?,
                    name: row.get(1)?,
                    set_code: row.get(2)?,
                    collector_number: row.get(3)?,
                    rarity: row.get(4)?,
                    oracle_text: row.get(5)?,
                    mana_cost: row.get(6)?,
                    card_type: row.get(7)?,
                    scryfall_id: row.get(8)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
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
    )>,
> {
    let mut statement = db.connection.prepare(
        "SELECT o.id,o.printing_id,o.quantity,o.language,o.foil,o.condition,o.notes,
                p.name,p.set_code,p.collector_number,p.mana_cost,p.card_type
         FROM owned_cards o JOIN printings p ON p.id=o.printing_id
         ORDER BY p.name COLLATE NOCASE",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            OwnedCard {
                id: row.get(0)?,
                printing_id: row.get(1)?,
                quantity: row.get(2)?,
                language: row.get(3)?,
                foil: row.get::<_, i64>(4)? != 0,
                condition: row.get(5)?,
                notes: row.get(6)?,
            },
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
            row.get(11)?,
        ))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
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
    )>,
> {
    let pattern = format!("%{}%", query.trim());
    let mut statement = db.connection.prepare(
        "SELECT o.id,o.printing_id,o.quantity,o.language,o.foil,o.condition,o.notes,
                p.name,p.set_code,p.collector_number,p.mana_cost,p.card_type
         FROM owned_cards o JOIN printings p ON p.id=o.printing_id
         WHERE p.name LIKE ?1 COLLATE NOCASE
            OR p.set_code LIKE ?1 COLLATE NOCASE
            OR p.collector_number LIKE ?1 COLLATE NOCASE
         ORDER BY p.name COLLATE NOCASE",
    )?;
    let rows = statement.query_map([pattern], |row| {
        Ok((
            OwnedCard {
                id: row.get(0)?,
                printing_id: row.get(1)?,
                quantity: row.get(2)?,
                language: row.get(3)?,
                foil: row.get::<_, i64>(4)? != 0,
                condition: row.get(5)?,
                notes: row.get(6)?,
            },
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
            row.get(11)?,
        ))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_owned(db: &Database, id: &str) -> AppResult<()> {
    let deleted = db
        .connection
        .execute("DELETE FROM owned_cards WHERE id=?1", [id])?;
    if deleted == 0 {
        return Err(AppError::NotFound(id.into()));
    }
    Ok(())
}

pub fn create_location(db: &Database, location: &Location) -> AppResult<()> {
    if location.name.trim().is_empty() {
        return Err(AppError::Validation("location name cannot be empty".into()));
    }
    db.connection.execute(
        "INSERT INTO locations VALUES (?1,?2,?3,?4)",
        params![
            location.id,
            location.name,
            location.kind,
            location.archived as i64
        ],
    )?;
    Ok(())
}

pub fn assign(
    db: &Database,
    card_id: &str,
    location_id: &str,
    quantity: i64,
    section: &str,
) -> AppResult<()> {
    let tx = db.connection.unchecked_transaction()?;
    let owned: i64 = tx
        .query_row(
            "SELECT quantity FROM owned_cards WHERE id=?1",
            [card_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(card_id.into()))?;
    if quantity <= 0 || quantity > owned {
        return Err(AppError::Validation("insufficient quantity".into()));
    }
    let _: String = tx
        .query_row("SELECT id FROM locations WHERE id=?1", [location_id], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or_else(|| AppError::NotFound(location_id.into()))?;
    tx.execute("INSERT INTO assignments VALUES (?1,?2,?3,?4) ON CONFLICT(owned_card_id) DO UPDATE SET location_id=excluded.location_id,section=excluded.section,quantity=excluded.quantity", params![card_id, location_id, section, quantity])?;
    tx.commit()?;
    Ok(())
}

pub fn create_tag(db: &Database, tag: &Tag) -> AppResult<()> {
    db.connection.execute(
        "INSERT INTO tags VALUES (?1,?2)",
        params![tag.id, tag.name.trim()],
    )?;
    Ok(())
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use crate::storage::Database;

    #[test]
    fn catalog_search_is_case_insensitive_and_limited() {
        let db = Database::in_memory().unwrap();
        upsert_catalog(
            &db,
            &[
                CatalogCard {
                    uuid: "one".into(),
                    name: "Lightning Bolt".into(),
                    set_code: "LEA".into(),
                    collector_number: "161".into(),
                    rarity: Some("common".into()),
                    oracle_text: None,
                    mana_cost: None,
                    card_type: None,
                    scryfall_id: None,
                },
                CatalogCard {
                    uuid: "two".into(),
                    name: "Lightning Helix".into(),
                    set_code: "RAV".into(),
                    collector_number: "220".into(),
                    rarity: Some("uncommon".into()),
                    oracle_text: None,
                    mana_cost: None,
                    card_type: None,
                    scryfall_id: None,
                },
            ],
            "test",
        )
        .unwrap();
        let results = search_catalog(&db, "LIGHTNING", 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Lightning Bolt");
    }

    #[test]
    fn catalog_search_treats_input_as_data() {
        let db = Database::in_memory().unwrap();
        let results = search_catalog(&db, "' OR 1=1 --", 25).unwrap();
        assert!(results.is_empty());
    }
}
