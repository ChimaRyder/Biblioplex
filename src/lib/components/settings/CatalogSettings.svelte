<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "../ui/Icon.svelte";
  import { Input } from "$lib/components/ui/input";
  import { invoke } from "@tauri-apps/api/core";

  let importPath = "";
  let importStatus = "";
  let importing = false;
  let clearing = false;
  let clearArmed = false;

  async function importMtgJson() {
    if (!importPath.trim() || importing) return;
    importing = true; importStatus = "Importing catalog…";
    try {
      const count = await invoke<number>("catalog_import_mtgjson", { path: importPath.trim() });
      importStatus = `Imported ${count.toLocaleString()} catalog cards. Your collection data was preserved.`;
    } catch (err) { importStatus = "Import failed: " + String(err); }
    finally { importing = false; }
  }

  async function clearCatalog() {
    if (!clearArmed) { clearArmed = true; importStatus = "Click Clear catalog again to confirm. Owned cards will be preserved."; return; }
    clearing = true; clearArmed = false; importStatus = "Clearing unused catalog records…";
    try { const count = await invoke<number>("catalog_clear"); importStatus = `Cleared ${count.toLocaleString()} unused catalog printings. Owned cards were preserved.`; }
    catch (err) { importStatus = "Catalog clear failed: " + String(err); }
    finally { clearing = false; }
  }
</script>

<section class="my-11 flex items-center justify-between gap-6 rounded-3xl border border-border bg-gradient-to-br from-[#161d35] to-[#101729] p-10 max-sm:p-6"><div><p class="mb-2 text-[11px] font-bold tracking-[.16em] text-[#8b9bbd]">APPLICATION SETTINGS</p><h2 class="mb-3 max-w-xl font-serif text-5xl leading-none tracking-tight max-sm:text-3xl">Keep your catalog current.</h2><p class="max-w-xl text-base leading-relaxed text-[#aab5ce]">Import card metadata locally so searching and compact collection mode remain available offline.</p></div><div class="text-7xl text-[#f7c873] max-sm:hidden" aria-hidden="true">⚙</div></section>
<section class="mb-5 max-w-3xl rounded-2xl border border-border bg-panel p-7 max-sm:p-5" aria-labelledby="catalog-title">
  <p class="mb-2 text-[11px] font-bold tracking-[.16em] text-[#8b9bbd]">CARD CATALOG</p><h2 id="catalog-title" class="mb-2 text-2xl tracking-tight">Import MTGJSON</h2>
  <p class="text-sm leading-relaxed text-muted">Use an absolute path to an AllPrintings.json file. Catalog refreshes update metadata without removing owned cards, notes, or organization.</p>
  <form class="mt-6 flex gap-3 max-sm:flex-col" on:submit|preventDefault={importMtgJson}>
    <label class="sr-only" for="catalog-path">AllPrintings JSON path</label>
    <Input id="catalog-path" bind:value={importPath} placeholder="/home/you/Downloads/AllPrintings.json" disabled={importing} />
    <Button type="submit" disabled={importing || !importPath.trim()}><Icon name="upload" size={15} />{importing ? "Importing…" : "Import catalog"}</Button>
  </form>
  {#if importStatus}<p class:import-error={importStatus.startsWith("Import failed")} class="mt-4 text-sm text-emerald-300" role="status">{importStatus}</p>{/if}
  <div class="mt-7 flex items-center justify-between gap-4 border-t border-border pt-5 max-sm:flex-col max-sm:items-start"><div><p class="text-sm font-semibold text-foreground">Reset imported catalog</p><p class="mt-1 text-xs text-muted">Removes unused catalog metadata, faces, and cached images. Owned cards remain safe.</p></div><Button variant="destructive" size="sm" disabled={clearing} onclick={clearCatalog}><Icon name="trash" size={14} />{clearArmed ? "Confirm clear" : clearing ? "Clearing…" : "Clear catalog"}</Button></div>
</section>
<section class="max-w-3xl rounded-2xl border border-border bg-panel p-7 max-sm:p-5"><p class="mb-2 text-[11px] font-bold tracking-[.16em] text-[#8b9bbd]">COMING LATER</p><h2 class="mb-2 text-xl">Images and backups</h2><p class="text-sm leading-relaxed text-muted">Optional cached images, versioned JSON export/import, and advanced SQLite backups will be added through the application command layer.</p></section>
