use crate::{
    domain::{Box, BoxEntry, Location, OwnedCard},
    error::{AppError, AppResult},
    storage::Database,
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Backup {
    pub format_version: u32,
    pub owned_cards: Vec<OwnedCard>,
    pub locations: Vec<Location>,
    #[serde(default)] pub boxes: Vec<Box>,
    #[serde(default)] pub box_entries: Vec<BoxEntry>,
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
    let boxes = db.connection.prepare("SELECT id,name,archived FROM boxes")?.query_map([], |r| Ok(Box { id:r.get(0)?, name:r.get(1)?, archived:r.get::<_,i64>(2)? != 0, entry_count: 0 }))?.collect::<Result<Vec<_>,_>>()?;
    let box_entries = db.connection.prepare("SELECT id,box_id,owned_card_id,printing_id,quantity FROM box_entries")?.query_map([], |r| Ok(BoxEntry { id:r.get(0)?, box_id:r.get(1)?, owned_card_id:r.get(2)?, printing_id:r.get(3)?, quantity:r.get(4)? }))?.collect::<Result<Vec<_>,_>>()?;
    serde_json::to_string(&Backup {
        format_version: 1,
        owned_cards: cards,
        locations,
        boxes,
        box_entries,
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
    for b in backup.boxes { tx.execute("INSERT INTO boxes(id,name,archived) VALUES (?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,archived=excluded.archived", params![b.id,b.name,b.archived as i64])?; }
    for card in backup.owned_cards {
        if card.quantity <= 0 {
            return Err(AppError::Validation("quantity must be positive".into()));
        }
        tx.execute("INSERT INTO owned_cards(id,printing_id,quantity,language,foil,condition,notes) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET printing_id=excluded.printing_id,quantity=excluded.quantity,language=excluded.language,foil=excluded.foil,condition=excluded.condition,notes=excluded.notes", params![card.id, card.printing_id, card.quantity, card.language, card.foil as i64, card.condition, card.notes])?;
    }
    for e in backup.box_entries { if e.quantity <= 0 { return Err(AppError::Validation("quantity must be positive".into())); } tx.execute("INSERT INTO box_entries(id,box_id,owned_card_id,printing_id,quantity) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET box_id=excluded.box_id,owned_card_id=excluded.owned_card_id,printing_id=excluded.printing_id,quantity=excluded.quantity", params![e.id,e.box_id,e.owned_card_id,e.printing_id,e.quantity])?; }
    tx.commit()?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TextImportResult { pub imported: i64, pub skipped: i64 }

fn valid_text(value: &str) -> bool { !value.is_empty() && !value.chars().any(|c| c.is_control()) }

pub fn export_text(db: &Database, format: &str) -> AppResult<String> {
    if format != "mtgo" && format != "mtga" { return Err(AppError::Validation("unsupported export format".into())); }
    let mut stmt = db.connection.prepare("SELECT o.quantity,p.name,p.set_code,p.collector_number FROM owned_cards o JOIN printings p ON p.id=o.printing_id ORDER BY p.name COLLATE NOCASE, p.set_code, p.collector_number")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?)))?;
    let mut lines = Vec::new();
    for row in rows {
        let (quantity, name, set_code, collector) = row?;
        if quantity > 0 && valid_text(&name) && valid_text(&set_code) && valid_text(&collector) {
            lines.push(if format == "mtga" { format!("{} {} ({}) {}", quantity, name, set_code, collector) } else { format!("{} {}", quantity, name) });
        }
    }
    Ok(lines.join("\n"))
}

pub fn import_text(db: &Database, input: &str) -> AppResult<TextImportResult> {
    if input.trim().is_empty() { return Err(AppError::Validation("import text cannot be empty".into())); }
    let tx = db.connection.unchecked_transaction()?;
    let mut imported = 0;
    let mut skipped = 0;
    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() { continue; }
        let mut parts = line.splitn(2, char::is_whitespace);
        let quantity: i64 = match parts.next().and_then(|v| v.parse().ok()) { Some(v) if v > 0 => v, _ => { skipped += 1; continue; } };
        let rest = match parts.next().map(str::trim).filter(|v| valid_text(v)) { Some(v) => v, None => { skipped += 1; continue; } };
        let (name, set_code, collector) = if let Some(close) = rest.rfind(") ") {
            if let Some(open) = rest[..close].rfind(" (") {
                let collector = rest[close + 2..].trim();
                let set_code = rest[open + 2..close].trim();
                let name = rest[..open].trim();
                if valid_text(name) && valid_text(set_code) && valid_text(collector) { (name, Some(set_code), Some(collector)) } else { skipped += 1; continue; }
            } else { (rest, None, None) }
        } else { (rest, None, None) };
        let printing_id: Option<String> = if let (Some(set), Some(number)) = (set_code, collector) {
            tx.query_row("SELECT id FROM printings WHERE name=?1 COLLATE NOCASE AND set_code=?2 COLLATE NOCASE AND collector_number=?3 ORDER BY id LIMIT 1", params![name, set, number], |r| r.get(0)).optional()?
        } else {
            tx.query_row("SELECT p.id FROM printings p LEFT JOIN owned_cards o ON o.printing_id=p.id WHERE p.name=?1 COLLATE NOCASE GROUP BY p.id ORDER BY CASE WHEN count(o.id)>0 THEN 0 ELSE 1 END, p.id LIMIT 1", [name], |r| r.get(0)).optional()?
        };
        let Some(printing_id) = printing_id else { skipped += 1; continue; };
        let existing: Option<(String, i64)> = tx.query_row("SELECT id,quantity FROM owned_cards WHERE printing_id=?1 AND language='en' AND foil=0 AND condition='near_mint' ORDER BY id LIMIT 1", [&printing_id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
        if let Some((id, current)) = existing { tx.execute("UPDATE owned_cards SET quantity=?2 WHERE id=?1", params![id, current + quantity])?; }
        else { tx.execute("INSERT INTO owned_cards(id,printing_id,quantity,language,foil,condition,notes) VALUES (?1,?2,?3,'en',0,'near_mint',NULL)", params![uuid::Uuid::new_v4().to_string(), printing_id, quantity])?; }
        imported += 1;
    }
    tx.commit()?;
    Ok(TextImportResult { imported, skipped })
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
                mana_cost: None, mana_value: None, colors: Vec::new(),
                card_type: None,
                power: None,
                toughness: None,
                scryfall_id: None,
                faces: Vec::new(),
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
                mana_cost: None, mana_value: None, colors: Vec::new(),
                card_type: None,
                power: None,
                toughness: None,
                scryfall_id: None,
                faces: Vec::new(),
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
