<script lang="ts">
  import {
    ArrowUpDown,
    Box,
    ChevronDown,
    Filter,
    Grid2X2,
    Package,
    Plus,
    Search,
    Settings,
    Sparkles,
    Trash2,
    Upload,
    X,
  } from "@lucide/svelte";

  export let name: "grid" | "box" | "deck" | "settings" | "search" | "plus" | "trash" | "x" | "upload" | "chevronDown" | "filter" | "arrowUpDown" | "sparkles";
  export let size = 16;
  export let strokeWidth = 1.8;

  const icons = { grid: Grid2X2, box: Box, deck: Package, settings: Settings, search: Search, plus: Plus, trash: Trash2, x: X, upload: Upload, chevronDown: ChevronDown, filter: Filter, arrowUpDown: ArrowUpDown, sparkles: Sparkles };
  $: IconComponent = icons[name];
</script>

<svelte:component this={IconComponent} {size} {strokeWidth} aria-hidden="true" />
