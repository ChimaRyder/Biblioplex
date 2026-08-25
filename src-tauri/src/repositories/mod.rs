use crate::storage::Database;
use crate::{
    domain::{CatalogCard, CatalogFace, Location, OwnedCard, Tag},
    error::{AppError, AppResult},
};
use rusqlite::{params, OptionalExtension};

pub fn upsert_catalog(db: &Database, cards: &[CatalogCard], version: &str) -> AppResult<()> {
    let tx = db.connection.unchecked_transaction()?;
    for card in cards {
        tx.execute("INSERT INTO printings(id,name,set_code,collector_number,rarity,oracle_text,mana_cost,card_type,power,toughness,scryfall_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET name=excluded.name,set_code=excluded.set_code,collector_number=excluded.collector_number,rarity=excluded.rarity,oracle_text=excluded.oracle_text,mana_cost=excluded.mana_cost,card_type=excluded.card_type,power=excluded.power,toughness=excluded.toughness,scryfall_id=excluded.scryfall_id",
            params![card.uuid, card.name, card.set_code, card.collector_number, card.rarity, card.oracle_text, card.mana_cost, card.card_type, card.power, card.toughness, card.scryfall_id])?;
        tx.execute("DELETE FROM card_faces WHERE printing_id=?1", [&card.uuid])?;
        let logical_faces = if card.faces.is_empty() {
            vec![CatalogFace { face_order: 0, name: card.name.clone(), mana_cost: card.mana_cost.clone(), card_type: card.card_type.clone(), oracle_text: card.oracle_text.clone(), power: card.power.clone(), toughness: card.toughness.clone(), scryfall_id: card.scryfall_id.clone(), cached_path: None, image_status: "missing".into() }]
        } else { card.faces.clone() };
        for face in &logical_faces {
            tx.execute("INSERT INTO card_faces(printing_id,face_order,name,mana_cost,card_type,oracle_text,power,toughness,scryfall_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![card.uuid, face.face_order, face.name, face.mana_cost, face.card_type, face.oracle_text, face.power, face.toughness, face.scryfall_id])?;
        }
    }
    tx.execute("INSERT INTO catalog_metadata(id,version,imported_at) VALUES (1,?1,datetime('now')) ON CONFLICT(id) DO UPDATE SET version=excluded.version, imported_at=excluded.imported_at", [version])?;
    tx.commit()?;
    Ok(())
}

pub fn clear_catalog(db: &Database) -> AppResult<i64> {
    let tx = db.connection.unchecked_transaction()?;
    tx.execute("DELETE FROM image_cache_entries WHERE printing_id IN (SELECT p.id FROM printings p WHERE NOT EXISTS (SELECT 1 FROM owned_cards o WHERE o.printing_id=p.id))", [])?;
    tx.execute("DELETE FROM card_faces WHERE printing_id IN (SELECT p.id FROM printings p WHERE NOT EXISTS (SELECT 1 FROM owned_cards o WHERE o.printing_id=p.id))", [])?;
    let deleted = tx.execute("DELETE FROM printings WHERE NOT EXISTS (SELECT 1 FROM owned_cards o WHERE o.printing_id=printings.id)", [])? as i64;
    tx.execute("DELETE FROM catalog_metadata", [])?;
    tx.commit()?;
    Ok(deleted)
}

pub fn search_catalog(db: &Database, query: &str, limit: i64) -> AppResult<Vec<CatalogCard>> {
    let limit = limit.clamp(1, 100);
    let pattern = format!("%{}%", query.trim());
    let mut statement = db.connection.prepare(
        "SELECT id,name,set_code,collector_number,rarity,oracle_text,mana_cost,card_type,power,toughness,scryfall_id
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
            power: row.get(8)?, toughness: row.get(9)?, scryfall_id: row.get(10)?,
            faces: Vec::new(),
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

pub fn find_owned_by_printing(db: &Database, printing_id: &str) -> AppResult<Vec<OwnedCard>> {
    let mut statement = db.connection.prepare(
        "SELECT id,printing_id,quantity,language,foil,condition,notes FROM owned_cards WHERE printing_id=?1 ORDER BY id",
    )?;
    let rows = statement.query_map([printing_id], |row| Ok(OwnedCard {
        id: row.get(0)?, printing_id: row.get(1)?, quantity: row.get(2)?,
        language: row.get(3)?, foil: row.get::<_, i64>(4)? != 0,
        condition: row.get(5)?, notes: row.get(6)?,
    }))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn find_catalog_card(db: &Database, printing_id: &str) -> AppResult<Option<CatalogCard>> {
    db.connection
        .query_row(
            "SELECT id,name,set_code,collector_number,rarity,oracle_text,mana_cost,card_type,power,toughness,scryfall_id
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
                    power: row.get(8)?, toughness: row.get(9)?, scryfall_id: row.get(10)?,
                    faces: Vec::new(),
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
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<CatalogFace>,
    )>,
> {
    let mut statement = db.connection.prepare(
        "SELECT o.id,o.printing_id,o.quantity,o.language,o.foil,o.condition,o.notes,
                p.name,p.set_code,p.collector_number,p.mana_cost,p.card_type,p.rarity,p.oracle_text,p.power,p.toughness,p.scryfall_id
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
            row.get(12)?,
            row.get(13)?,
            row.get(14)?,
            row.get(15)?, row.get(16)?,
        ))
    })?;
    let rows = rows.collect::<Result<Vec<_>, _>>()?;
    rows.into_iter().map(|row| { let faces = faces_for_printing(db, &row.0.printing_id)?; Ok((row.0,row.1,row.2,row.3,row.4,row.5,row.6,row.7,row.8,row.9,row.10,faces)) }).collect()
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
        Vec<CatalogFace>,
    )>,
> {
    let pattern = format!("%{}%", query.trim());
    let mut statement = db.connection.prepare(
        "SELECT o.id,o.printing_id,o.quantity,o.language,o.foil,o.condition,o.notes,
                p.name,p.set_code,p.collector_number,p.mana_cost,p.card_type,p.rarity,p.oracle_text,p.power,p.toughness,p.scryfall_id
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
            row.get(12)?,
            row.get(13)?,
            row.get(14)?,
            row.get(15)?, row.get(16)?,
        ))
    })?;
    let rows = rows.collect::<Result<Vec<_>, _>>()?;
    rows.into_iter().map(|row| { let faces = faces_for_printing(db, &row.0.printing_id)?; Ok((row.0,row.1,row.2,row.3,row.4,row.5,row.6,row.7,row.8,row.9,row.10,faces)) }).collect()
}

pub fn faces_for_printing(db: &Database, printing_id: &str) -> AppResult<Vec<CatalogFace>> {
    let mut statement = db.connection.prepare("SELECT f.face_order,f.name,f.mana_cost,f.card_type,f.oracle_text,f.power,f.toughness,f.scryfall_id,i.cached_path,COALESCE(i.status,'missing') FROM card_faces f LEFT JOIN image_cache_entries i ON i.printing_id=f.printing_id AND i.face_order=f.face_order WHERE f.printing_id=?1 ORDER BY f.face_order")?;
    let rows = statement.query_map([printing_id], |row| Ok(CatalogFace { face_order: row.get(0)?, name: row.get(1)?, mana_cost: row.get(2)?, card_type: row.get(3)?, oracle_text: row.get(4)?, power: row.get(5)?, toughness: row.get(6)?, scryfall_id: row.get(7)?, cached_path: row.get(8)?, image_status: row.get(9)? }))?;
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

pub fn update_owned(db: &Database, id: &str, quantity: i64, language: &str, foil: bool, condition: &str, notes: Option<&str>) -> AppResult<()> {
    if quantity <= 0 { return Err(AppError::Validation("quantity must be positive".into())); }
    let assigned: i64 = db.connection.query_row("SELECT COALESCE(quantity, 0) FROM assignments WHERE owned_card_id=?1", [id], |row| row.get(0)).unwrap_or(0);
    if quantity < assigned { return Err(AppError::Validation(format!("quantity cannot be less than the {assigned} assigned card(s)"))); }
    let changed = db.connection.execute("UPDATE owned_cards SET quantity=?2,language=?3,foil=?4,condition=?5,notes=?6 WHERE id=?1", params![id, quantity, language, foil as i64, condition, notes])?;
    if changed == 0 { return Err(AppError::NotFound(id.into())); }
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
                    power: None,
                    toughness: None,
                    faces: Vec::new(),
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
                    power: None,
                    toughness: None,
                    faces: Vec::new(),
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
