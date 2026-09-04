use crate::{domain::{CatalogFace, OwnedCard}, error::AppError, repositories, services, storage::Database};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct BoxView { pub id: String, pub name: String, pub archived: bool }
#[derive(Debug, Serialize)]
pub struct BoxEntryView { pub id: String, pub box_id: String, pub owned_card_id: Option<String>, pub printing_id: String, pub quantity: i64, pub name: String, pub set_code: String, pub collector_number: String, pub rarity: Option<String>, pub mana_cost: Option<String>, pub mana_value: Option<f64>, pub card_type: Option<String>, pub colors: Vec<String>, pub collection_quantity: i64 }

#[tauri::command]
pub fn list_boxes(state: State<'_, Mutex<Database>>, archived: bool) -> Result<Vec<BoxView>, String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::list_boxes(&db,archived).map_err(db_error).map(|v|v.into_iter().map(|b|BoxView{id:b.id,name:b.name,archived:b.archived}).collect()) }
#[tauri::command]
pub fn create_box(state: State<'_, Mutex<Database>>, name: String) -> Result<BoxView, String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; let id=uuid::Uuid::new_v4().to_string(); services::create_independent_box(&db,&id,&name).map_err(db_error)?; Ok(BoxView{id,name:name.trim().into(),archived:false}) }
#[tauri::command]
pub fn update_box(state: State<'_, Mutex<Database>>, id: String, name: String) -> Result<(), String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::rename_box(&db,&id,&name).map_err(db_error) }
#[tauri::command]
pub fn archive_box(state: State<'_, Mutex<Database>>, id: String, archived: bool) -> Result<(), String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::archive_box(&db,&id,archived).map_err(db_error) }
#[tauri::command]
pub fn delete_box(state: State<'_, Mutex<Database>>, id: String) -> Result<(), String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::delete_box(&db,&id).map_err(db_error) }
#[tauri::command]
pub fn list_box_entries(state: State<'_, Mutex<Database>>, box_id: String, query: String) -> Result<Vec<BoxEntryView>, String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::list_box_entries(&db,&box_id,&query).map_err(db_error).map(|v|v.into_iter().map(|(e,c)|{ let collection_quantity=db.connection.query_row("SELECT COALESCE(SUM(quantity),0) FROM owned_cards WHERE printing_id=?1", [&e.printing_id], |r| r.get(0)).unwrap_or(0); BoxEntryView{id:e.id,box_id:e.box_id,owned_card_id:e.owned_card_id.clone(),printing_id:e.printing_id,name:c.name,set_code:c.set_code,collector_number:c.collector_number,rarity:c.rarity,mana_cost:c.mana_cost,mana_value:c.mana_value,card_type:c.card_type,colors:c.colors,quantity:e.quantity,collection_quantity} }).collect()) }
#[tauri::command]
pub fn add_box_entry(state: State<'_, Mutex<Database>>, box_id: String, owned_card_id: Option<String>, printing_id: String, quantity: i64) -> Result<(), String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::add_box_entry(&db,&crate::domain::BoxEntry{id:uuid::Uuid::new_v4().to_string(),box_id,owned_card_id,printing_id,quantity}).map_err(db_error) }
#[tauri::command]
pub fn update_box_entry(state: State<'_, Mutex<Database>>, id: String, quantity: i64) -> Result<(), String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::update_box_entry(&db,&id,quantity).map_err(db_error) }
#[tauri::command]
pub fn delete_box_entry(state: State<'_, Mutex<Database>>, id: String) -> Result<(), String> { let db=state.lock().map_err(|_| "database lock poisoned".to_string())?; services::delete_box_entry(&db,&id).map_err(db_error) }

#[derive(Debug, Serialize)]
pub struct OwnedCardView {
    pub id: String,
    pub name: String,
    pub set_code: String,
    pub collector_number: String,
    pub mana_cost: Option<String>,
    pub mana_value: Option<f64>,
    pub colors: Vec<String>,
    pub card_type: Option<String>,
    pub quantity: i64,
    pub language: String,
    pub foil: bool,
    pub condition: String,
    pub notes: Option<String>,
    pub rarity: Option<String>,
    pub oracle_text: Option<String>,
    pub scryfall_id: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub faces: Vec<CardFaceView>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CardFaceView {
    pub face_order: i64,
    pub name: String,
    pub mana_cost: Option<String>,
    pub card_type: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub image: ImageView,
}
#[derive(Debug, Serialize, Clone)]
pub struct ImageView { pub cached_path: Option<String>, pub remote_url: Option<String>, pub status: String }

fn face_views(faces: Vec<CatalogFace>, base: (&str, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)) -> Vec<CardFaceView> {
    let source = if faces.is_empty() { vec![CatalogFace { face_order: 0, name: base.0.into(), mana_cost: base.1, card_type: base.2, oracle_text: base.3, power: base.4, toughness: base.5, scryfall_id: None, cached_path: None, image_status: "missing".into() }] } else { faces };
    let fallback_id = source.first().and_then(|face| face.scryfall_id.clone());
    source.into_iter().map(|face| {
        let image_id = face.scryfall_id.clone().or_else(|| fallback_id.clone());
        let remote_url = image_id.as_ref().and_then(|id| {
            let mut chars = id.chars();
            let side = if face.face_order == 0 { "front" } else { "back" };
            Some(format!("https://cards.scryfall.io/normal/{}/{}/{}/{}.jpg", side, chars.next()?, chars.next()?, id))
        });
        CardFaceView { face_order: face.face_order, name: face.name, mana_cost: face.mana_cost, card_type: face.card_type, oracle_text: face.oracle_text, power: face.power, toughness: face.toughness, image: ImageView { cached_path: face.cached_path, remote_url, status: face.image_status } }
    }).collect()
}

#[derive(Debug, Deserialize)]
pub struct AddCardRequest {
    pub name: String,
    pub set_code: String,
    pub collector_number: String,
    pub quantity: i64,
    pub language: String,
    pub foil: bool,
    pub condition: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CatalogSearchRequest {
    pub query: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct OwnedSearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct CatalogCardView {
    pub uuid: String,
    pub name: String,
    pub set_code: String,
    pub collector_number: String,
    pub rarity: Option<String>,
    pub oracle_text: Option<String>,
    pub mana_cost: Option<String>,
    pub colors: Vec<String>,
    pub card_type: Option<String>,
    pub scryfall_id: Option<String>,
    pub collection_quantity: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddCatalogCardRequest {
    pub printing_id: String,
    pub quantity: i64,
    pub language: String,
    pub foil: bool,
    pub condition: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DuplicateOwnedCardView { pub id: String, pub quantity: i64, pub language: String, pub foil: bool, pub condition: String, pub notes: Option<String> }

#[tauri::command]
pub fn find_owned_catalog_cards(state: State<'_, Mutex<Database>>, printing_id: String) -> Result<Vec<DuplicateOwnedCardView>, String> {
    let db = state.lock().map_err(|_| "database lock poisoned".to_string())?;
    services::find_owned_by_printing(&db, printing_id.trim()).map_err(db_error).map(|cards| cards.into_iter().map(|card| DuplicateOwnedCardView { id: card.id, quantity: card.quantity, language: card.language, foil: card.foil, condition: card.condition, notes: card.notes }).collect())
}

#[derive(Debug, Deserialize)]
pub struct UpdateOwnedCardRequest { pub id: String, pub quantity: i64, pub language: String, pub foil: bool, pub condition: String, pub notes: Option<String> }

fn db_error(error: AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub fn check_image_provider() -> Result<String, String> {
    crate::integrations::scryfall::probe()
        .map(|_| "stable".to_string())
        .map_err(|_| "unavailable".to_string())
}

#[tauri::command]
pub fn list_owned_cards(state: State<'_, Mutex<Database>>) -> Result<Vec<OwnedCardView>, String> {
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    services::list_owned(&db).map_err(db_error).map(|rows| {
        rows.into_iter()
            .map(
                |(card, name, set_code, collector_number, mana_cost, mana_value, card_type, rarity, oracle_text, power, toughness, scryfall_id, faces)| OwnedCardView {
                    id: card.id,
                    name: name.clone(),
                    set_code,
                    collector_number,
                    mana_cost: mana_cost.clone(),
                    mana_value,
                    colors: repositories::colors_for_printing(&db, &card.printing_id).unwrap_or_default(),
                    card_type: card_type.clone(),
                    quantity: card.quantity,
                    language: card.language,
                    foil: card.foil,
                    condition: card.condition,
                    notes: card.notes,
                    rarity, oracle_text: oracle_text.clone(), power: power.clone(), toughness: toughness.clone(), scryfall_id,
                    faces: face_views(faces, (&name, mana_cost.clone(), card_type.clone(), oracle_text.clone(), power.clone(), toughness.clone())),
                },
            )
            .collect()
    })
}

#[tauri::command]
pub fn search_owned_cards(
    state: State<'_, Mutex<Database>>,
    request: OwnedSearchRequest,
) -> Result<Vec<OwnedCardView>, String> {
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let rows = if request.query.trim().is_empty() {
        services::list_owned(&db)
    } else {
        services::search_owned(&db, &request.query)
    }
    .map_err(db_error)?;
    Ok(rows
        .into_iter()
        .map(
            |(card, name, set_code, collector_number, mana_cost, mana_value, card_type, rarity, oracle_text, power, toughness, scryfall_id, faces)| OwnedCardView {
                id: card.id,
                name: name.clone(),
                set_code,
                collector_number,
                mana_cost: mana_cost.clone(),
                mana_value,
                colors: repositories::colors_for_printing(&db, &card.printing_id).unwrap_or_default(),
                card_type: card_type.clone(),
                quantity: card.quantity,
                language: card.language,
                foil: card.foil,
                condition: card.condition,
                notes: card.notes,
                rarity, oracle_text: oracle_text.clone(), power: power.clone(), toughness: toughness.clone(), scryfall_id,
                faces: face_views(faces, (&name, mana_cost.clone(), card_type.clone(), oracle_text.clone(), power.clone(), toughness.clone())),
            },
        )
        .collect())
}

#[tauri::command]
pub fn add_owned_card(
    state: State<'_, Mutex<Database>>,
    request: AddCardRequest,
) -> Result<OwnedCardView, String> {
    if request.name.trim().is_empty() || request.set_code.trim().is_empty() {
        return Err("card name and set code are required".into());
    }
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let printing_id = uuid::Uuid::new_v4().to_string();
    repositories::upsert_catalog(
        &db,
        &[crate::domain::CatalogCard {
            uuid: printing_id.clone(),
            name: request.name.trim().into(),
            set_code: request.set_code.trim().into(),
            collector_number: request.collector_number.trim().into(),
            rarity: None,
            oracle_text: None,
            mana_cost: None,
            mana_value: None,
            colors: Vec::new(),
            card_type: None,
            scryfall_id: None,
            power: None,
            toughness: None,
            faces: Vec::new(),
        }],
        "manual",
    )
    .map_err(db_error)?;
    let card = OwnedCard {
        id: uuid::Uuid::new_v4().to_string(),
        printing_id,
        quantity: request.quantity,
        language: request.language,
        foil: request.foil,
        condition: request.condition,
        notes: request.notes,
    };
    services::add_owned_card(&db, &card).map_err(db_error)?;
    Ok(OwnedCardView {
        id: card.id,
        name: request.name.trim().into(),
        set_code: request.set_code.trim().into(),
        collector_number: request.collector_number.trim().into(),
        mana_cost: None,
        mana_value: None,
        colors: Vec::new(),
        card_type: None,
        quantity: card.quantity,
        language: card.language,
        foil: card.foil,
        condition: card.condition,
        notes: card.notes,
        rarity: None,
        oracle_text: None,
        scryfall_id: None,
        power: None,
        toughness: None,
        faces: vec![CardFaceView { face_order: 0, name: request.name.trim().into(), mana_cost: None, card_type: None, oracle_text: None, power: None, toughness: None, image: ImageView { cached_path: None, remote_url: None, status: "missing".into() } }],
    })
}

#[tauri::command]
pub fn add_owned_catalog_card(
    state: State<'_, Mutex<Database>>,
    request: AddCatalogCardRequest,
) -> Result<OwnedCardView, String> {
    if request.printing_id.trim().is_empty() {
        return Err("a catalog printing must be selected".into());
    }
    if request.quantity <= 0 {
        return Err("quantity must be greater than zero".into());
    }
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let catalog = repositories::find_catalog_card(&db, &request.printing_id)
        .map_err(db_error)?
        .ok_or_else(|| "selected catalog printing does not exist".to_string())?;
    let card = OwnedCard {
        id: uuid::Uuid::new_v4().to_string(),
        printing_id: catalog.uuid.clone(),
        quantity: request.quantity,
        language: request.language,
        foil: request.foil,
        condition: request.condition,
        notes: request.notes,
    };
    services::add_owned_catalog_card(&db, &card).map_err(db_error)?;
    Ok(OwnedCardView {
        id: card.id,
        name: catalog.name.clone(),
        set_code: catalog.set_code,
        collector_number: catalog.collector_number,
        mana_cost: catalog.mana_cost.clone(),
        mana_value: catalog.mana_value,
        colors: catalog.colors.clone(),
        card_type: catalog.card_type.clone(),
        quantity: card.quantity,
        language: card.language,
        foil: card.foil,
        condition: card.condition,
        notes: card.notes,
        rarity: catalog.rarity,
        oracle_text: catalog.oracle_text.clone(),
        scryfall_id: catalog.scryfall_id,
        power: catalog.power.clone(),
        toughness: catalog.toughness.clone(),
        faces: face_views(Vec::new(), (&catalog.name, catalog.mana_cost.clone(), catalog.card_type.clone(), catalog.oracle_text.clone(), catalog.power.clone(), catalog.toughness.clone())),
    })
}

#[tauri::command]
pub fn remove_owned_card(state: State<'_, Mutex<Database>>, id: String) -> Result<(), String> {
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    services::remove_owned_card(&db, &id).map_err(db_error)
}

#[tauri::command]
pub fn remove_owned_cards(state: State<'_, Mutex<Database>>, ids: Vec<String>) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    services::remove_owned_cards(&db, &ids).map_err(db_error)
}

#[tauri::command]
pub fn update_owned_card(state: State<'_, Mutex<Database>>, request: UpdateOwnedCardRequest) -> Result<(), String> {
    let db = state.lock().map_err(|_| "database lock poisoned".to_string())?;
    services::update_owned_card(&db, &request.id, request.quantity, request.language.trim(), request.foil, request.condition.trim(), request.notes.as_deref().map(str::trim).filter(|value| !value.is_empty())).map_err(db_error)
}

#[tauri::command]
pub fn catalog_import_mtgjson(
    state: State<'_, Mutex<Database>>,
    path: String,
) -> Result<usize, String> {
    let source = Path::new(&path);
    let input = std::fs::read_to_string(source)
        .map_err(|error| format!("could not read MTGJSON file: {error}"))?;
    let cards = crate::integrations::mtgjson::parse_all_printings(&input).map_err(db_error)?;
    let version = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mtgjson")
        .to_string();
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    services::import_catalog(&db, &cards, &version).map_err(db_error)?;
    Ok(cards.len())
}

#[tauri::command]
pub fn catalog_import_mtgjson_text(state: State<'_, Mutex<Database>>, input: String) -> Result<usize, String> {
    let cards = crate::integrations::mtgjson::parse_all_printings(&input).map_err(db_error)?;
    let db = state.lock().map_err(|_| "database lock poisoned".to_string())?;
    services::import_catalog(&db, &cards, "file upload").map_err(db_error)?;
    Ok(cards.len())
}

#[tauri::command]
pub fn choose_catalog_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("MTGJSON catalog", &["json"])
        .set_title("Choose AllPrintings.json")
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn catalog_clear(state: State<'_, Mutex<Database>>) -> Result<i64, String> {
    let db = state.lock().map_err(|_| "database lock poisoned".to_string())?;
    repositories::clear_catalog(&db).map_err(db_error)
}

#[tauri::command]
pub fn export_collection_text(state: State<'_, Mutex<Database>>, format: String) -> Result<String, String> {
    let db = state.lock().map_err(|_| "database lock poisoned".to_string())?;
    crate::backup::export_text(&db, format.trim()).map_err(db_error)
}

#[tauri::command]
pub fn import_collection_text(state: State<'_, Mutex<Database>>, input: String) -> Result<crate::backup::TextImportResult, String> {
    let db = state.lock().map_err(|_| "database lock poisoned".to_string())?;
    crate::backup::import_text(&db, &input).map_err(db_error)
}

#[tauri::command]
pub fn catalog_search(
    state: State<'_, Mutex<Database>>,
    request: CatalogSearchRequest,
) -> Result<Vec<CatalogCardView>, String> {
    let query = request.query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let db = state
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    services::search_catalog(&db, query, request.limit.unwrap_or(25))
        .map_err(db_error)
        .map(|cards| {
            cards
                .into_iter()
                .map(|card| CatalogCardView {
                    uuid: card.uuid.clone(),
                    name: card.name,
                    set_code: card.set_code,
                    collector_number: card.collector_number,
                    rarity: card.rarity,
                    oracle_text: card.oracle_text,
                    mana_cost: card.mana_cost,
                    colors: card.colors,
                    card_type: card.card_type,
                    scryfall_id: card.scryfall_id,
                    collection_quantity: db.connection.query_row("SELECT COALESCE(SUM(quantity),0) FROM owned_cards WHERE printing_id=?1", [&card.uuid], |r| r.get(0)).unwrap_or(0),
                })
                .collect()
        })
}
