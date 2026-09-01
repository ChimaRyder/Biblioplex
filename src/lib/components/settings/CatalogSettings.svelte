<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "../ui/Icon.svelte";
  import Textarea from "$lib/components/ui/textarea/textarea.svelte";
  import * as Dialog from "$lib/components/ui/dialog"; 
  import * as Select from "$lib/components/ui/select";
  import { invoke } from "@tauri-apps/api/core"; 
  import { toast } from "svelte-sonner";

  let importPath="", catalogFile: File | undefined, status="", importing=false, clearing=false, clearArmed=false, format="mtgo", importOpen=false, text="", importingText=false;
  async function chooseCatalog(){if(importing)return; try { const selectedPath=await invoke<string|null>("choose_catalog_file"); if(!selectedPath)return; importPath=selectedPath; importing=true; status="Importing catalog…"; const n=await invoke<number>("catalog_import_mtgjson",{path:selectedPath}); status=`Imported ${n.toLocaleString()} catalog cards.`; toast.success(`Imported ${n.toLocaleString()} MTGJSON catalog cards.`); } catch(e) { status="Import failed: "+String(e); toast.error(`MTGJSON import failed: ${String(e)}`); } finally { importing=false; } }
  async function clearCatalog(){if(!clearArmed){clearArmed=true;status="Click Clear catalog again to confirm. Owned cards will be preserved.";return} clearing=true;clearArmed=false;try{const n=await invoke<number>("catalog_clear");status=`Cleared ${n.toLocaleString()} catalog cards.`;toast.success(`Cleared ${n.toLocaleString()} unused catalog printings.`)}catch(e){status="Catalog clear failed: "+String(e);toast.error(`Catalog clear failed: ${String(e)}`)}finally{clearing=false}}
  async function exportCollection(){try{const value=await invoke<string>("export_collection_text",{format});const url=URL.createObjectURL(new Blob([value],{type:"text/plain;charset=utf-8"}));const a=document.createElement("a");a.href=url;a.download=`collection-${format}.txt`;a.click();URL.revokeObjectURL(url);toast.success(`Exported collection as ${format.toUpperCase()}.`)}catch(e){toast.error(`Export failed: ${String(e)}`)}}
  async function importCollection(){if(!text.trim()||importingText)return;importingText=true;try{const r=await invoke<{imported:number;skipped:number}>("import_collection_text",{input:text});importOpen=false;text="";toast.success(`Imported ${r.imported} card${r.imported===1?"":"s"}${r.skipped?`. Skipped ${r.skipped}`:""}.`);window.dispatchEvent(new CustomEvent("collection-imported"))}catch(e){toast.error(`Import failed: ${String(e)}`)}finally{importingText=false}}
</script>

<section class="mb-5 w-full rounded-2xl border border-border bg-panel p-7 max-sm:p-5">
  <div class="flex items-center justify-between gap-6 max-sm:flex-col max-sm:items-stretch">
    <div class="min-w-0">
      <h2 class="text-md font-semibold">Import MTGJSON</h2>
      <p class="text-xs leading-relaxed text-muted">Biblioplex uses <a href="https://mtgjson.com/" target="_blank" class="underline hover:no-underline">MTGJSON</a> for local catalog management.</p>
      {#if importPath}<p class="mt-3 truncate text-sm text-muted" title={importPath}>{importPath}</p>{/if}
    </div>
    <Button class="shrink-0" onclick={chooseCatalog} disabled={importing}>
      <Icon name="upload" size={15}/>
      {importing?"Importing…":"Import MTGJSON"}
    </Button>
  </div>
    {#if status}
    <p class="mt-4 text-sm text-status-synced" role="status">{status}</p>
    {/if}
    <div class="mt-7 flex items-center justify-between border-t border-border pt-5">
      <div>
        <p class="text-md font-semibold">Reset Catalog</p>
        <p class="mt-1 text-xs text-muted">Collection data will be retained, but the catalog will be lost.</p>
      </div>
      <Button variant="destructive" size="lg" disabled={clearing} onclick={clearCatalog}>
        <Icon name="trash" size={14}/>
        {clearArmed?"Confirm" : clearing? "Clearing…" : "Clear"}
      </Button>
    </div>
  </section>

<section class="w-full rounded-2xl border border-border bg-panel p-7 max-sm:p-5">
  <div class="flex items-center justify-between gap-6 max-sm:flex-col max-sm:items-stretch">
    <div>
      <h2 class="text-md font-semibold">Import/Export</h2>
      <p class="mt-1 text-xs text-muted">Export your collection in various formats or import cards from external sources.</p>
    </div>
    <div class="flex items-center gap-3 max-sm:flex-col max-sm:items-stretch">
      <Select.Root type="single" bind:value={format}>
        <Select.Trigger class="w-32 h-full"aria-label="Export format">{format === "mtgo" ? "MTGO" : "MTGA"}</Select.Trigger>
        <Select.Content>
          <Select.Item value="mtgo" label="MTGO">MTGO</Select.Item>
          <Select.Item value="mtga" label="MTGA">MTGA</Select.Item>
        </Select.Content>
      </Select.Root>
      <Button size="lg" onclick={exportCollection}><Icon name="download" size={15}/>Export</Button>
      <Button variant="outline" size="lg" onclick={()=>importOpen=true}><Icon name="upload" size={15}/>Import</Button>
    </div>
  </div>
</section>

<Dialog.Root bind:open={importOpen}>
  <Dialog.Content class="w-[calc(100vw-2rem)] max-w-5xl border-border bg-panel-raised text-foreground sm:max-w-5xl">
    <Dialog.Header>
      <Dialog.Title>Import Collection</Dialog.Title>
    </Dialog.Header>
    <Textarea class="min-h-80 resize-y max-h-180 font-mono text-sm" bind:value={text} placeholder={'4 Lightning Bolt\n4 Sol Ring (CMM) 396'} aria-label="Collection import text"/>
    <Dialog.Footer class="bg-panel border-border">
      <Dialog.Close class="inline-flex h-8 items-center justify-center rounded-lg border border-border bg-background px-2.5 text-sm font-medium text-foreground transition-colors outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50">Cancel</Dialog.Close>
      <Button onclick={importCollection} disabled={importingText||!text.trim()}>
        {importingText?"Importing…":"Import"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
