# MTG Local-First Card Collection Manager

## Summary

The MTG Local-First Card Collection Manager is an open-source, account-free desktop application for managing Magic: The Gathering collections. It works offline by default, with MTGJSON-powered catalog imports and optional Scryfall card images when a network connection is available.

The application prioritizes local ownership, physical organization, data portability, and user control. It does not require accounts, subscriptions, cloud hosting, or mandatory telemetry.

## Goals

- Maintain a searchable local inventory of owned cards.
- Organize physical cards into user-defined Boxes and assembled Decks.
- Support user-defined Tags and filtering.
- Provide an offline compact view and an optional image-based card grid.
- Keep user data portable through versioned export and import.
- Preserve collection usability when the network, MTGJSON, or Scryfall is unavailable.

## Non-goals for the MVP

Scanning, OCR, marketplace pricing, authentication, cloud hosting, peer-to-peer synchronization, and mobile clients are deferred until after the MVP.

## Product MVPs

### MVP 1: Local collection foundation

- Add, edit, remove, and search owned cards.
- Track individual physical copies with stable local IDs.
- Store card metadata locally using versioned MTGJSON imports.
- Support quantity, printing/set, language, foil status, condition, and notes.
- Provide an offline compact list view.
- Persist user data in SQLite.
- Provide JSON export/import and optional SQLite backup.

### MVP 2: Organization

- Create, rename, archive, and delete Boxes.
- Assign each physical card copy to exactly one current location.
- Move cards between Boxes and Decks.
- Prevent a physical copy from being allocated to multiple locations.
- Display Box contents with quantity totals and filters.

### MVP 3: Deck management

- Create, rename, archive, and delete owned assembled Decks.
- Add owned copies to Decks while respecting available quantities.
- Support Mainboard, Sideboard, and Commander sections.
- Show insufficient quantities before assignment and reject invalid allocations.
- Display Decks in compact mode and image-grid mode when images are available.

### MVP 4: Tags and presentation

- Create, rename, merge, and delete user-defined Tags.
- Apply multiple Tags to owned cards.
- Filter the collection, Boxes, and Decks by Tags.
- Provide compact offline and image-grid presentation modes.
- Clearly indicate unavailable images and stale metadata.

## Recommended architecture

### Technology baseline

- **Desktop shell:** Tauri.
- **Frontend:** TypeScript and Svelte.
- **Application backend:** Rust.
- **Persistence:** SQLite with explicit migrations.
- **External card data:** MTGJSON through an isolated catalog importer.
- **Images:** Local filesystem cache indexed by Scryfall card and image identifiers.

The frontend communicates with Rust through typed application commands. UI components must not access SQLite or external HTTP services directly.

### Architectural boundaries

- `CardCatalogRepository`: canonical card and printing metadata.
- `CollectionRepository`: owned cards, quantities, and user annotations.
- `LocationRepository`: Boxes, Decks, sections, and assignments.
- `TagRepository`: Tags and card-tag relationships.
- `AllocationService`: validates availability, movement, and deck assignment.
- `MTGJSONImporter`: imports and projects catalog metadata into SQLite.
- `ScryfallImageProvider`: retrieves optional images while respecting provider limits.
- `BackupService`: validates, exports, imports, and restores versioned user data.

Domain rules belong in Rust services rather than frontend components. All mutating operations must be transactional.

### Data flow

1. The user invokes an action in the Svelte UI.
2. A typed command is sent to the Rust application layer.
3. The relevant service validates the operation and executes a repository transaction.
4. The updated state is returned to the UI.
5. Optional MTGJSON or Scryfall network work runs only through its adapter and never blocks offline collection operations.

### Lightweight catalog strategy

MTGJSON is an import source, not the runtime database. The importer must:

- Accept the complete dataset, split set files, or another supported MTGJSON distribution.
- Import only the fields required by the application.
- Normalize cards and printings into the local SQLite catalog.
- Preserve MTGJSON UUIDs and Scryfall identifiers as external references.
- Avoid loading the entire source dataset into memory when a streaming or incremental approach is practical.
- Remove temporary source archives after a successful import unless the user chooses to retain them.

The application must not require the raw MTGJSON JSON files for normal searching or offline operation.

## Core data model

All user-created records use stable UUIDs. MTGJSON UUIDs and Scryfall identifiers are external references and must never be used as primary keys for user data.

### Entities

- **Card:** canonical card identity and normalized metadata.
- **Printing:** set, collector number, rarity, artist, and printing-specific data.
- **OwnedCard:** a physical copy, or an explicitly grouped quantity of identical copies.
- **Box:** a user-defined physical storage location.
- **Deck:** an assembled owned deck.
- **DeckSection:** Mainboard, Sideboard, or Commander.
- **Tag:** a user-defined label.
- **CardTag:** many-to-many relationship between owned cards and Tags.
- **LocationAssignment:** the current Box or Deck location of an owned card.
- **SyncMetadata:** source, import version, timestamps, and image-cache status.

### Ownership and allocation rules

- Every owned card has one current location: a Box or a Deck.
- A card assigned to a Deck is unavailable for assignment to another location unless moved first.
- Deck assignment cannot exceed the available owned quantity.
- Catalog updates may update metadata but must not remove user annotations, Tags, locations, or Deck assignments.
- Removing catalog metadata must not silently delete an owned-card record.
- Archived Boxes and Decks retain their contents until explicitly removed.

## Functional requirements

### Offline behavior

- The application must work without an account or network connection.
- Compact mode must be fully usable offline.
- Existing collection data must remain readable when MTGJSON or Scryfall is unavailable.
- Card-grid mode must show cached images when present and a clear placeholder when absent.
- Network failures must not corrupt local data or block local operations.

### Card catalog and images

- MTGJSON catalog imports must be repeatable and idempotent.
- Catalog imports must project only the fields required by the application.
- Catalog updates must not require retaining the original MTGJSON source archive.
- Catalog updates must preserve all user-owned records and annotations.
- Images must be cached locally and associated with stable external identifiers.
- Scryfall image requests must respect provider rate limits and attribution requirements.
- The UI must distinguish current, stale, and unavailable metadata or images.

### Backup and portability

- Export must produce a versioned JSON format containing all user-owned data and required catalog references.
- Import must validate the format and schema before changing the database.
- Invalid or incompatible imports must leave existing data unchanged.
- Import/export must preserve stable UUIDs, Tags, locations, and Deck assignments.
- A raw SQLite backup may be offered as an advanced recovery option.

## Non-functional requirements

- Common searches and local reads should remain responsive for collections of at least 100,000 owned cards.
- Database writes must be transactional.
- Every schema change must use an explicit migration.
- No mandatory telemetry, account, subscription, or remote service.
- User data and cached assets must use documented platform-specific application directories.
- The project must select and document an open-source license before implementation begins.
- The architecture must leave room for a future synchronization service without requiring one in the MVP.

## Testing and acceptance criteria

### Unit tests

- Quantity and availability validation.
- Moving cards between Boxes and Decks.
- Deck section assignment.
- Tag creation, application, merge, and deletion.
- Archive and restore behavior.

### Repository and service tests

- Database migrations.
- Transaction rollback behavior.
- Idempotent catalog imports.
- Catalog updates preserving user state.
- JSON export/import round trips.
- Rejection of invalid imports without data mutation.

### Provider tests

- MTGJSON parsing and field projection using recorded fixtures.
- Repeatable catalog updates without duplicate records.
- Scryfall image-provider retry and rate-limit behavior.
- Image-cache hit, miss, stale, and unavailable cases.
- Offline behavior with no provider connectivity.

### UI and end-to-end scenarios

1. Import catalog data, disconnect from the network, and browse/search cards.
2. Add owned copies, place them in Boxes, and move them into a Deck.
3. Attempt an over-allocation and verify that it is rejected with a clear message.
4. Apply multiple Tags and filter by them.
5. Display a Deck in compact mode while offline.
6. Display a Deck in grid mode with cached and unavailable images.
7. Export data, create a clean database, import it, and verify equivalent state.
8. Update or remove catalog data and verify that user-owned records remain intact.

## Future extension points

The following are deliberately outside the MVP but should remain possible through the architecture:

- Additional card-data providers.
- Barcode or camera-assisted entry.
- OCR and scanning workflows.
- Pricing integrations.
- Optional encrypted cloud or peer-to-peer synchronization.
- Mobile clients sharing the export format and domain model.
- Additional deck sections and format validation.

## Explicit assumptions

- The first release targets Windows, macOS, and Linux desktop systems.
- MTGJSON is the initial external card-data source.
- Scryfall is an optional image provider only; it is not required for metadata, search, or compact mode.
- MVP synchronization is limited to local export/import.
- Each physical card copy has one current location: a Box or a Deck.
- Decks represent assembled owned decks and consume available collection quantities.
- The initial implementation supports English card data first while leaving room for multilingual metadata.
