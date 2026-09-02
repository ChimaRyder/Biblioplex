use rusqlite::{Connection, Result};

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn in_memory() -> Result<Self> {
        let db = Self {
            connection: Connection::open_in_memory()?,
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn open(path: &std::path::Path) -> Result<Self> {
        let db = Self {
            connection: Connection::open(path)?,
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn migrate(&self) -> Result<()> {
        self.connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS catalog_metadata (id INTEGER PRIMARY KEY CHECK (id = 1), version TEXT NOT NULL, imported_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS printings (id TEXT PRIMARY KEY, name TEXT NOT NULL, set_code TEXT NOT NULL, collector_number TEXT NOT NULL, rarity TEXT, oracle_text TEXT, mana_cost TEXT, mana_value REAL, card_type TEXT, scryfall_id TEXT, colors TEXT);
             CREATE TABLE IF NOT EXISTS card_faces (printing_id TEXT NOT NULL REFERENCES printings(id) ON DELETE CASCADE, face_order INTEGER NOT NULL, name TEXT NOT NULL, mana_cost TEXT, card_type TEXT, oracle_text TEXT, power TEXT, toughness TEXT, scryfall_id TEXT, PRIMARY KEY(printing_id, face_order));
             CREATE TABLE IF NOT EXISTS image_cache_entries (id TEXT PRIMARY KEY, printing_id TEXT NOT NULL REFERENCES printings(id) ON DELETE CASCADE, face_order INTEGER NOT NULL DEFAULT 0, cached_path TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'cached', updated_at TEXT NOT NULL DEFAULT (datetime('now')));
             CREATE INDEX IF NOT EXISTS idx_printings_name ON printings(name);
             CREATE TABLE IF NOT EXISTS owned_cards (id TEXT PRIMARY KEY, printing_id TEXT NOT NULL REFERENCES printings(id), quantity INTEGER NOT NULL CHECK(quantity > 0), language TEXT NOT NULL, foil INTEGER NOT NULL, condition TEXT NOT NULL, notes TEXT);
             CREATE TABLE IF NOT EXISTS locations (id TEXT PRIMARY KEY, name TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('box','deck')), archived INTEGER NOT NULL DEFAULT 0);
             CREATE TABLE IF NOT EXISTS assignments (owned_card_id TEXT PRIMARY KEY REFERENCES owned_cards(id) ON DELETE CASCADE, location_id TEXT NOT NULL REFERENCES locations(id), section TEXT NOT NULL DEFAULT 'box', quantity INTEGER NOT NULL CHECK(quantity > 0));
             CREATE TABLE IF NOT EXISTS tags (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE);
             CREATE TABLE IF NOT EXISTS owned_card_tags (owned_card_id TEXT NOT NULL REFERENCES owned_cards(id) ON DELETE CASCADE, tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE, PRIMARY KEY(owned_card_id, tag_id));",
        )?;
        for (name, definition) in [
            (
                "mana_cost",
                "ALTER TABLE printings ADD COLUMN mana_cost TEXT",
            ),
            (
                "card_type",
                "ALTER TABLE printings ADD COLUMN card_type TEXT",
            ),
            ("mana_value", "ALTER TABLE printings ADD COLUMN mana_value REAL"),
            ("power", "ALTER TABLE printings ADD COLUMN power TEXT"),
            ("toughness", "ALTER TABLE printings ADD COLUMN toughness TEXT"),
            ("colors", "ALTER TABLE printings ADD COLUMN colors TEXT"),
        ] {
            let exists: bool = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_table_info('printings') WHERE name=?1)",
                [name],
                |row| row.get(0),
            )?;
            if !exists {
                self.connection.execute(definition, [])?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    #[test]
    fn migrations_create_runtime_schema() {
        let db = Database::in_memory().unwrap();
        let count: i64 = db
            .connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(count >= 6);
    }
}
