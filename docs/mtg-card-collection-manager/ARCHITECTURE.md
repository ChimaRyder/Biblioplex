# Project Architecture and Structure

## Purpose

This document defines the implementation architecture for the MTG Local-First Card Collection Manager described in README.md.

The project is a lightweight Tauri desktop application with a Svelte frontend, a Rust application core, and SQLite persistence. MTGJSON is used only as an import source for card metadata. Scryfall is an optional image provider and must never be required for collection management or compact offline mode.

## Architectural principles

1. **Local-first:** user data is stored and operated on locally.
2. **Offline-capable:** catalog search, collection management, Boxes, Decks, Tags, and compact mode work without network access.
3. **Explicit boundaries:** UI, domain logic, persistence, importers, and image services remain independently testable.
4. **User-data safety:** catalog refreshes never overwrite user-owned state.
5. **Portable data:** user records use stable local UUIDs and a versioned export format.
6. **Lightweight runtime:** raw MTGJSON files and image assets are not required to run the app.
7. **Replaceable integrations:** MTGJSON and Scryfall are adapters, not dependencies of the domain model.
8. **Transactional mutations:** operations that change ownership, allocation, or organization commit atomically.

## System context

    ┌────────────────────────────────────────────────────────────┐
    │                    Tauri Desktop Application               │
    │                                                            │
    │  ┌────────────────┐     typed commands     ┌─────────────┐ │
    │  │ Svelte frontend│ ◄────────────────────► │ Rust core   │ │
    │  │ views + state  │                         │ services    │ │
    │  └────────────────┘                         └──────┬──────┘ │
    │                                                    │        │
    │                                      ┌─────────────┼──────┐ │
    │                                      │             │      │ │
    │                                  SQLite       MTGJSON  Scryfall│
    │                                  database     importer  images │
    │                                      │             │      │ │
    │                                  user data   catalog   cache  │
    └────────────────────────────────────────────────────────────┘

The frontend never connects directly to SQLite or external services. All application commands pass through the Rust core.

## Repository layout

    .
    ├── docs/
    │   └── mtg-card-collection-manager/
    │       ├── README.md
    │       └── ARCHITECTURE.md
    ├── src/
    │   ├── app/
    │   │   ├── App.svelte
    │   │   ├── routes.ts
    │   │   └── stores/
    │   ├── components/
    │   │   ├── cards/
    │   │   ├── collection/
    │   │   ├── boxes/
    │   │   ├── decks/
    │   │   ├── tags/
    │   │   └── shared/
    │   ├── features/
    │   │   ├── catalog/
    │   │   ├── collection/
    │   │   ├── locations/
    │   │   ├── decks/
    │   │   ├── tags/
    │   │   ├── images/
    │   │   └── backup/
    │   ├── lib/
    │   │   ├── commands/
    │   │   ├── types/
    │   │   ├── validation/
    │   │   └── formatting/
    │   └── main.ts
    ├── src-tauri/
    │   ├── Cargo.toml
    │   ├── tauri.conf.json
    │   ├── migrations/
    │   ├── src/
    │   │   ├── main.rs
    │   │   ├── commands/
    │   │   ├── domain/
    │   │   ├── repositories/
    │   │   ├── services/
    │   │   ├── integrations/
    │   │   │   ├── mtgjson/
    │   │   │   └── scryfall/
    │   │   ├── backup/
    │   │   ├── storage/
    │   │   └── error.rs
    │   └── tests/
    ├── fixtures/
    │   ├── mtgjson/
    │   └── scryfall/
    ├── tests/
    │   ├── e2e/
    │   └── fixtures/
    ├── package.json
    ├── svelte.config.js
    ├── tsconfig.json
    └── README.md

The exact frontend build tooling may change, but the boundary between src/ and src-tauri/ is architectural: frontend code presents state and invokes commands; Rust owns business rules and persistence.

## Frontend structure

src/app/ owns navigation, layout, global application state, and startup/loading states. It does not contain domain rules.

Each feature module contains its screens, view models, command wrappers, and feature-specific presentation logic:

- catalog/: card search, printing selection, and catalog-import status.
- collection/: owned-card list, editing, compact view, and collection filters.
- locations/: Box management and location movement.
- decks/: Deck management, Deck sections, allocation UI, and Deck views.
- tags/: Tag management and tag filters.
- images/: image-grid state, cache status, placeholders, and retry actions.
- backup/: export/import workflow and validation feedback.

Frontend types mirror the public command DTOs from Rust. They must not expose database implementation details.

## UI design system and iconography

### Design-system decision

The frontend uses shadcn-svelte as its component foundation, with Tailwind CSS for utility styling. Supported controls must use the corresponding shadcn-svelte components provided by the dependency and exposed through the project’s configured local component paths. Feature code may compose these components, but must not replace them with hand-authored equivalents. See the shadcn-svelte documentation at https://shadcn-svelte.com/docs.

Use the current Svelte 5 and Tailwind CSS 4-compatible shadcn-svelte setup when initializing the design system. Keep the generated component configuration, CSS variables, utility helper, and dependency-provided component source under version control. Components must be added through the shadcn-svelte component workflow rather than invented locally.

### Required UI packages

- shadcn-svelte: accessible, locally owned UI component source.
- bits-ui: headless interaction primitives used by shadcn-svelte components.
- tailwindcss: utility styling and design tokens.
- tailwind-variants: component variants where generated shadcn-svelte components require them.
- lucide-svelte: general-purpose interface icons.

Lucide icons are for application chrome and actions only: navigation, search, settings, sorting, filtering, import, remove, close, refresh, and status indicators. Import icons as individual Svelte components so unused icons are not rendered or bundled unnecessarily.

### MTG-specific symbols

The Mana font from mana-master remains separate from the general icon system:

- Use Mana symbols for mana costs and Magic card symbols.
- Use Lucide for generic application controls.
- Do not replace Mana glyphs with Lucide icons.
- Keep the mana-master source directory ignored.
- Commit only the specific font assets required by the application, currently public/fonts/mana.woff2.
- Keep the local Mana CSS mapping small and limited to symbols rendered by the app.

### Component organization

UI primitives belong under src/lib/components/ui/ and must come from the shadcn-svelte component set:

- button
- input
- select
- command
- popover
- dropdown-menu
- dialog
- sheet
- table
- badge
- tooltip

The following component policy is mandatory:

- Use shadcn-svelte components whenever a matching primitive exists, including Button, Input, Select, Command, Popover, DropdownMenu, Dialog, Sheet, Table, Badge, Tooltip, and related components.
- Import feature-facing primitives through the project-owned shadcn component paths under `src/lib/components/ui/`.
- Do not create custom replacements for shadcn-svelte primitives, wrappers that duplicate their behavior, or bespoke controls with equivalent responsibilities.
- Feature components may compose shadcn-svelte primitives and add domain-specific layout/content, but may not reimplement primitive interaction, accessibility, focus management, keyboard navigation, animation, or state behavior.
- If shadcn-svelte does not provide a required primitive, document the exception in the feature architecture before introducing another dependency or a narrowly scoped custom component.

Application composition belongs under feature and layout directories:

- src/lib/components/layout/Sidebar.svelte
- src/lib/components/layout/PageHeader.svelte
- src/lib/components/collection/CollectionTable.svelte
- src/lib/components/collection/CollectionSearch.svelte
- src/lib/components/collection/QuickAdd.svelte
- src/lib/components/catalog/CatalogSearchResults.svelte

Feature components may compose UI primitives, but UI primitives must not import feature services, repositories, or Tauri commands.

### Visual and accessibility rules

- Use design tokens for colors, spacing, radius, typography, and focus rings rather than one-off values.
- Preserve the existing dark Biblioplex visual language through shadcn-svelte theme variables.
- Every icon-only button must have an accessible label and a tooltip where the action is not obvious.
- Icons must not be the sole carrier of destructive or ambiguous meaning.
- Keyboard navigation and visible focus states are required for search results, tables, menus, dialogs, and Quick Add.
- Use Mana icon color classes only for mana symbols; application icons use the neutral UI palette.
- Prefer shadcn-svelte Command, Popover, DropdownMenu, and Table primitives for catalog search, filters, sorting, and collection presentation.

### Migration rule

Existing hand-authored CSS may remain only for non-component global concerns during the transition. New interactive controls must use shadcn-svelte primitives, and existing bespoke controls must be replaced feature-by-feature. Tailwind utilities should provide feature layout and composition; `app.css` is limited to Tailwind setup, theme tokens, resets, and Mana asset declarations. Keep Mana font declarations and glyph mappings in the global style layer because they are asset-specific, not general UI components.

## Rust core structure

src-tauri/src/commands/ contains the only functions exposed to the frontend. Commands parse request DTOs, resolve services, execute operations, and return stable response DTOs or typed application errors. Commands must remain thin and contain no SQL.

src-tauri/src/domain/ contains framework-independent entities, value objects, enums, and invariants for Cards, Printings, OwnedCards, Boxes, Decks, DeckSections, Tags, assignments, and freshness states. It must not depend on Tauri, HTTP clients, or UI types.

src-tauri/src/services/ coordinates domain operations:

- AllocationService: moves owned cards and validates available quantities.
- CollectionService: creates and edits owned-card records.
- LocationService: manages Boxes and Decks.
- TagService: manages Tags and card-tag relationships.
- CatalogService: coordinates catalog import and refresh.
- ImageService: resolves image requests and cache behavior.
- BackupService: performs validated export and import.

src-tauri/src/repositories/ provides CardCatalogRepository, CollectionRepository, LocationRepository, TagRepository, and ImageCacheRepository. Repositories translate between SQLite rows and domain types; cross-aggregate workflows belong in services.

## Persistence design

SQLite is the single runtime database. It contains both imported catalog and user-owned data, separated logically and through foreign-key relationships.

Catalog tables:

- cards
- printings
- card_faces for double-faced or multi-faced cards
- catalog_metadata for MTGJSON version, import timestamp, and source checksum

User tables:

- owned_cards
- boxes
- decks
- deck_sections
- location_assignments
- tags
- owned_card_tags

Cache tables:

- image_cache_entries

Catalog rows use MTGJSON UUIDs and Scryfall IDs as indexed external identifiers. User rows use generated stable UUIDs. Foreign keys must be enabled.

## Allocation model

The MVP treats an owned_cards record as a physical card or grouped quantity of identical physical cards. Independent Box entries reference owned cards or catalog printings without consuming inventory; location assignments are reserved for physical-location and Deck allocation workflows.

The allocation service enforces non-negative quantities, available quantity limits, atomic movement, and valid Deck sections. Box membership is intentionally independent and may be duplicated across Boxes.

## MTGJSON import pipeline

    source file or download
            ↓
    format detection and checksum validation
            ↓
    streaming parser
            ↓
    field projection and normalization
            ↓
    staging tables
            ↓
    transactional catalog upsert
            ↓
    catalog version record
            ↓
    temporary source cleanup

The importer must use staging tables or an equivalent isolated transaction so a failed import cannot leave a partially updated catalog.

Only fields needed for search, compact display, printing selection, and future legality checks should be imported initially. The raw source archive is optional and is not required at runtime.

Imports must be idempotent. Repeating an import must not create duplicates. New catalog data may update metadata but must not alter user-owned annotations, Tags, locations, or Deck allocations.

## Scryfall image pipeline

1. The UI requests an image using a stored Scryfall ID or image reference.
2. ScryfallImageProvider checks the local cache.
3. A cache miss may trigger a rate-limited network request.
4. The image is written to the cache atomically.
5. The UI receives the cached image or an unavailable-image state.

Image downloads must never be part of adding a card, searching compact mode, importing MTGJSON, or opening the database. Users must be able to disable image retrieval and clear the cache.

## Backup and import/export

The portable JSON format contains an export format version, application metadata, user-owned cards, Boxes, Decks, Deck sections, assignments, Tags, relationships, and catalog references required to reconnect owned cards.

The export should not embed the full MTGJSON catalog or image cache. Missing catalog records on import must be represented safely and may be repaired by a later catalog refresh.

Import behavior:

1. Read the file without modifying the live database.
2. Validate version, required fields, UUIDs, relationships, and quantities.
3. Present validation errors before commit.
4. Import inside one transaction.
5. Roll back all changes if any operation fails.

## Public command API

- catalog.search
- catalog.importMtgJson
- catalog.getStatus
- collection.list
- collection.create
- collection.update
- collection.delete
- allocation.move
- boxes.list, boxes.create, boxes.update, boxes.archive
- decks.list, decks.create, decks.update, decks.archive
- decks.assignCard, decks.removeCard
- tags.list, tags.create, tags.update, tags.merge, tags.delete, tags.apply
- images.get, images.clearCache
- backup.export, backup.import

Errors use stable codes such as not_found, insufficient_quantity, invalid_assignment, catalog_unavailable, import_invalid, and image_unavailable.

## Testing and performance

Domain tests live beside Rust domain modules. Repository tests use temporary SQLite databases and migrations. MTGJSON tests use small fixtures for normal, malformed, duplicate, and multi-faced cards. Scryfall tests use recorded HTTP fixtures and never require live network access. Frontend and end-to-end tests cover offline startup, allocation constraints, catalog refresh, image fallback, and backup round trips.

Search queries must use indexed columns and pagination. Large catalog imports must not require retaining raw datasets. UI lists use pagination or virtualization, and image grids lazy-load thumbnails. Catalog refreshes run as background work with visible progress where practical.

## Future compatibility

The architecture leaves room for additional metadata providers, future synchronization using stable user UUIDs, mobile or web clients sharing the export format, individual-copy tracking, additional Deck sections, and format legality services. These extensions must not introduce accounts, remote services, or image dependencies into the MVP.
