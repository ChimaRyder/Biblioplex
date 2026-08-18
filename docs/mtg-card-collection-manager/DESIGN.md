---
name: Aether Vault
colors:
  surface: '#121414'
  surface-dim: '#16130f'
  surface-bright: '#3c3934'
  surface-container-lowest: '#100e0a'
  surface-container-low: '#1e1b17'
  surface-container: '#221f1b'
  surface-container-high: '#2d2925'
  surface-container-highest: '#38342f'
  on-surface: '#e9e1da'
  on-surface-variant: '#d0c5b5'
  inverse-surface: '#e9e1da'
  inverse-on-surface: '#33302b'
  outline: '#998f81'
  outline-variant: '#4d463a'
  surface-tint: '#e4c285'
  primary: '#ffdea6'
  on-primary: '#412d00'
  primary-container: '#e4c285'
  on-primary-container: '#674f1d'
  inverse-primary: '#745a28'
  secondary: '#c6c6c7'
  on-secondary: '#2f3131'
  secondary-container: '#454747'
  on-secondary-container: '#b4b5b5'
  tertiary: '#dbe2ff'
  on-tertiary: '#222f51'
  tertiary-container: '#b9c6f0'
  on-tertiary-container: '#455276'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#ffdea5'
  primary-fixed-dim: '#e4c285'
  on-primary-fixed: '#271900'
  on-primary-fixed-variant: '#5a4312'
  secondary-fixed: '#e2e2e2'
  secondary-fixed-dim: '#c6c6c7'
  on-secondary-fixed: '#1a1c1c'
  on-secondary-fixed-variant: '#454747'
  tertiary-fixed: '#dae2ff'
  tertiary-fixed-dim: '#b9c6ef'
  on-tertiary-fixed: '#0b1a3b'
  on-tertiary-fixed-variant: '#394669'
  background: '#16130f'
  on-background: '#e9e1da'
  surface-variant: '#38342f'
  mana-blue: '#0E68AB'
  mana-red: '#D3202A'
  mana-green: '#00733E'
  mana-black: '#150B00'
  status-synced: '#2DD4BF'
  status-offline: '#F59E0B'
typography:
  display-lg:
    fontFamily: EB Garamond
    fontSize: 48px
    fontWeight: '600'
    lineHeight: 56px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: EB Garamond
    fontSize: 32px
    fontWeight: '500'
    lineHeight: 40px
  card-title:
    fontFamily: EB Garamond
    fontSize: 20px
    fontWeight: '600'
    lineHeight: 24px
  body-lg:
    fontFamily: Geist
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-sm:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  data-label:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
    letterSpacing: 0.05em
  sync-status:
    fontFamily: JetBrains Mono
    fontSize: 10px
    fontWeight: '700'
    lineHeight: 12px
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 40px
  container-margin: 32px
  gutter: 16px
---

## Brand & Style
This design system is crafted for a high-end digital archive and trading card management interface. The personality is authoritative, immersive, and arcane, yet balanced by technological precision. It evokes the feeling of interacting with a private, secure terminal for rare artifacts.

The aesthetic follows a **Dark-Mode Glassmorphism** approach. It utilizes deep, obsidian surfaces layered with translucent panels that mimic the "spark" and ethereal nature of arcane lore. High-contrast typography ensures that complex data remains legible, while tactile feedback and subtle animations evoke the feeling of handling physical items in a digital space.

## Colors
The palette is rooted in a "Void, Gold, and Light" hierarchy, optimized for deep immersion and high legibility.

- **Primary:** A metallic gold (#e4c285) used for rare highlights, active states, and critical call-to-action elements.
- **Secondary:** Pure White (#FFFFFF), used to provide sharp contrast against the obsidian base. It is utilized for secondary interactive elements, icons, and navigational prominence.
- **Backgrounds:** A tiered system of obsidian and charcoal surfaces provides the foundation for the dark interface.
- **Mana Accents:** Functional UI elements like chips or filters utilize a traditional pentad (White, Blue, Black, Red, Green) to maintain thematic consistency.
- **Status:** A vibrant teal indicates "Synced" states, while a muted amber is reserved for "Offline" or "Local-only" indicators.

## Typography
The typographic strategy balances the "Arcane" (Serif) with the "Terminal" (Sans/Mono).

- **EB Garamond** is the display face, reserved for card titles, headers, and lore-heavy sections to provide a classical, premium feel.
- **Geist** is the primary interface face, providing a clean, modern contrast that balances the more ornate serif.
- **JetBrains Mono** is used for technical metadata, power/toughness, and data-heavy tables. This reinforces the database nature of the application.

For mobile devices, `display-lg` should scale down to 32px to ensure readability without excessive overflow.

## Layout & Spacing
The layout employs a **Dynamic Grid System** that adapts to the user's intent:

1.  **Art Mode (Visual-First):** A fluid masonry or "card-gallery" layout with generous 24px gutters. UI elements are minimized to let the card art dominate.
2.  **Data Mode (Utility-First):** A dense, structured table-view layout with 8px vertical spacing. This maximizes information density for collection auditing.

Navigation uses a fixed-width left sidebar (240px) on desktop. On mobile, the sidebar collapses into a bottom navigation bar, and the right-hand "Inspector" panel becomes a full-screen modal overlay.

## Elevation & Depth
Depth is established through **Luminance Layering** and **Glassmorphism** rather than traditional drop shadows.

- **Base Layer:** Solid Obsidian.
- **Panels:** Charcoal with a 1px inner border of 10% white to define edges.
- **Overlays:** Glassmorphic surfaces with a 40px backdrop blur and 60% opacity. A subtle gradient stroke (Top-Left: 20% white, Bottom-Right: 5% white) defines the boundary.
- **Interactions:** Hovering over a card triggers a "glow" elevation—a soft, primary gold outer bloom (20px blur, 15% opacity) to simulate internal illumination.

## Shapes
The design system utilizes **Soft (0.25rem)** roundedness to maintain a technical, professional aesthetic.

- **Cards:** Use `rounded-lg` (0.5rem) to mimic the physical geometry of card sleeves.
- **Input Fields/Buttons:** Use the base `rounded` (0.25rem) for a precise, sharp look.
- **Sync Badges:** Use `rounded-xl` (0.75rem) or full pill-shapes to distinguish them from standard functional UI components.

## Components
- **Buttons:** Primary buttons use a solid gold fill with black text. Secondary buttons use a solid white fill with black text or a white "ghost" style with a 1px border.
- **Sync Indicators:** Small pulsating dots in the top-right. White indicates a standard connection, while gold indicates a premium/high-speed sync state.
- **Input Fields:** Dark backgrounds with a bottom-only border. Upon focus, the border transitions to a primary gold glow.
- **Card Tiles:** In Art Mode, titles are overlaid on a dark-to-transparent gradient. In Data Mode, rows are separated by low-contrast outlines.
- **Tag Chips:** Rectangular labels using `data-label` typography. Active filters use a white background with black text to stand out against the dark UI.
- **Checkboxes & Radios:** Use the primary gold for checked states to provide high visual feedback.