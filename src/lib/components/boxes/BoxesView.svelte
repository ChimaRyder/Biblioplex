<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Command from "$lib/components/ui/command";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import * as Table from "$lib/components/ui/table";
  import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "$lib/components/ui/dropdown-menu";
  import Icon from "../ui/Icon.svelte";
  import CollectionFilters from "../collection/CollectionFilters.svelte";

  type Box = { id: string; name: string; archived: boolean };
  type Entry = { id: string; box_id: string; owned_card_id?: string; printing_id: string; quantity: number; name: string; set_code: string; collector_number: string; catalog_only: boolean; rarity?: string; mana_value?: number; card_type?: string; colors?: string[]; collection_quantity: number };
  type Catalog = { uuid: string; name: string; set_code: string; collector_number: string; rarity?: string; colors?: string[]; card_type?: string; mana_value?: number; collection_quantity?: number };

  let boxes: Box[] = [];
  let selected: Box | null = null;
  let entries: Entry[] = [];
  let query = "";
  let addQuery = "";
  let results: Catalog[] = [];
  let ownedResults: Entry[] = [];
  let source = "catalog";
  let newName = "";
  let loading = false;
  let error = "";
  let sortBy = "name";
  let sortAscending = true;
  let selectedColors = new Set<string>();
  let selectedTypes = new Set<string>();
  let selectedSets = new Set<string>();
  let quickOpen = false;
  let isEditingName = false;
  let nameDraft = "";
  let nameInput: HTMLInputElement;
  let searchTimer: ReturnType<typeof setTimeout> | undefined;

  $: availableSets = [...new Set(entries.map((entry) => entry.set_code))].sort();
  $: visibleEntries = entries
    .filter((entry) => {
      const colors = entry.colors ?? [];
      const type = entry.card_type?.split(/[—–-]/)[0].split(" ") ?? [];
      const colorMatch = !selectedColors.size || (selectedColors.has("Colorless") && !colors.length) || colors.some((color) => selectedColors.has(color.toUpperCase()));
      const typeMatch = !selectedTypes.size || [...selectedTypes].some((value) => type.includes(value));
      const setMatch = !selectedSets.size || selectedSets.has(entry.set_code);
      return colorMatch && typeMatch && setMatch;
    })
    .sort((a, b) => {
      if (sortBy === "quantity" || sortBy === "availability") {
        const aValue = sortBy === "quantity" ? a.quantity : a.collection_quantity;
        const bValue = sortBy === "quantity" ? b.quantity : b.collection_quantity;
        return sortAscending ? aValue - bValue : bValue - aValue;
      }
      const aValue = sortBy === "printing" ? `${a.set_code} ${a.collector_number}` : a.name;
      const bValue = sortBy === "printing" ? `${b.set_code} ${b.collector_number}` : b.name;
      const result = aValue.localeCompare(bValue);
      return sortAscending ? result : -result;
    });
  $: totalCards = visibleEntries.reduce((total, entry) => total + entry.quantity, 0);

  async function loadBoxes() {
    boxes = await invoke<Box[]>("list_boxes", { archived: false });
    if (!selected && boxes[0]) selected = boxes[0];
    if (selected) await loadEntries();
  }

  async function loadEntries() {
    if (!selected) return;
    loading = true;
    try {
      entries = await invoke<Entry[]>("list_box_entries", { boxId: selected.id, query });
    } catch (exception) {
      error = String(exception);
    } finally {
      loading = false;
    }
  }

  function searchAsYouType() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => loadEntries(), 180);
  }

  function clearBoxSearch() {
    if (searchTimer) clearTimeout(searchTimer);
    query = "";
    loadEntries();
  }

  async function create() {
    const name = newName.trim() || window.prompt("Create Box", "New Box")?.trim();
    if (!name) return;
    try { await invoke("create_box", { name }); newName = ""; await loadBoxes(); }
    catch (exception) { error = String(exception); }
  }

  async function removeBox() {
    if (!selected) return;
    await invoke("delete_box", { id: selected.id });
    selected = null; entries = []; await loadBoxes();
  }

  async function archive() {
    if (!selected) return;
    await invoke("archive_box", { id: selected.id, archived: true });
    selected = null; entries = []; await loadBoxes();
  }

  async function searchCatalog() {
    if (!addQuery.trim()) { results = []; ownedResults = []; return; }
    try {
      if (source === "catalog") results = await invoke<Catalog[]>("catalog_search", { request: { query: addQuery, limit: 8 } });
      else ownedResults = await invoke<Entry[]>("search_owned_cards", { request: { query: addQuery } });
    } catch (exception) { error = String(exception); }
  }

  async function add(card: Catalog) {
    if (!selected) return;
    await invoke("add_box_entry", { boxId: selected.id, ownedCardId: null, printingId: card.uuid, quantity: 1 });
    closeQuick(); await loadEntries();
  }

  async function addOwned(card: Entry) {
    if (!selected) return;
    await invoke("add_box_entry", { boxId: selected.id, ownedCardId: card.id, printingId: card.printing_id, quantity: 1 });
    closeQuick(); await loadEntries();
  }

  function closeQuick() { addQuery = ""; results = []; ownedResults = []; quickOpen = false; }
  async function remove(id: string) { await invoke("delete_box_entry", { id }); await loadEntries(); }
  async function startEditingName() { if (!selected) return; nameDraft = selected.name; isEditingName = true; await tick(); nameInput?.focus(); nameInput?.select(); }
  async function saveName() { if (!selected || !isEditingName) return; const name = nameDraft.trim(); if (!name || name === selected.name) { isEditingName = false; return; } await invoke("update_box", { id: selected.id, name }); selected = { ...selected, name }; boxes = boxes.map((box) => box.id === selected?.id ? { ...box, name } : box); isEditingName = false; }

  loadBoxes();
</script>

<div class="grid gap-6 lg:grid-cols-[220px_1fr]">
  <aside class="rounded-xl border border-border bg-panel p-4">
    <div class="mb-3 flex items-center justify-between">
      <h2 class="font-serif text-xl">Boxes</h2>
      <Button size="icon" class="size-8" aria-label="Create Box" onclick={create}><Icon name="plus" size={16} /></Button>
    </div>
    <div class="mb-3 flex gap-2"><Input class="h-8" bind:value={newName} placeholder="New box name" onkeydown={(event) => event.key === "Enter" && create()} /></div>
    {#each boxes as box}
      <button class={`mb-1 w-full rounded-md px-3 py-2 text-left text-sm ${selected?.id === box.id ? "bg-primary text-primary-foreground" : "hover:bg-accent"}`} onclick={() => { selected = box; loadEntries(); }}>{box.name}</button>
    {:else}<p class="text-xs text-muted">No boxes yet.</p>{/each}
  </aside>

  <section class="rounded-xl border border-border bg-panel p-6">
    <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div class="group flex min-w-0 items-center gap-2">
        {#if selected}
          {#if isEditingName}<Input bind:this={nameInput} bind:value={nameDraft} class="h-10 max-w-full font-serif text-3xl" onkeydown={(event) => event.key === "Enter" && saveName()} onblur={saveName} />
          {:else}<h2 class="font-serif text-3xl">{selected.name}</h2><span class="inline-flex min-w-7 items-center justify-center rounded-full bg-accent px-2 py-0.5 text-xs font-semibold">{totalCards}</span><Button variant="ghost" size="icon" class="size-7 opacity-0 group-hover:opacity-100" aria-label="Rename Box" onclick={startEditingName}><Icon name="pencil" size={15} /></Button>{/if}
        {:else}<h2 class="font-serif text-3xl">Select a Box</h2>{/if}
      </div>
      {#if selected}<DropdownMenu><DropdownMenuTrigger class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent" aria-label="Box settings"><Icon name="dots-three-vertical" size={18} /></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuItem onclick={() => {}}><Icon name="star" size={15} />Favorite</DropdownMenuItem><DropdownMenuItem onclick={archive}><Icon name="archive" size={15} />Archive</DropdownMenuItem><DropdownMenuItem class="text-destructive" onclick={removeBox}><Icon name="trash" size={15} />Delete</DropdownMenuItem></DropdownMenuContent></DropdownMenu>{/if}
    </div>

    {#if selected}
      <div class="mb-4 flex flex-wrap items-center gap-3">
        <div class="flex h-10 min-w-[260px] flex-1 items-center gap-2 rounded-md border border-border bg-background px-3 transition focus-within:border-gold focus-within:ring-2 focus-within:ring-gold/20 lg:mr-12">
          <Icon name="search" size={17} />
          <Input class="h-9! border-0! bg-transparent! shadow-none! outline-none! ring-0! focus:border-transparent! focus:ring-0! focus-visible:border-transparent! focus-visible:ring-0!" bind:value={query} placeholder="Search this box…" aria-label="Search this box" oninput={searchAsYouType} onkeydown={(event) => { if (event.key === "Enter") { if (searchTimer) clearTimeout(searchTimer); loadEntries(); } }} />
          {#if query}
            <button type="button" class="inline-flex size-7 shrink-0 items-center justify-center rounded-md text-muted transition hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-label="Clear Box search" title="Clear search" onclick={clearBoxSearch}>
              <Icon name="x" size={15} />
            </button>
          {/if}
        </div>
        <Select.Root type="single" bind:value={sortBy}><Select.Trigger class="h-10! min-w-32 rounded-md border-border bg-background px-3">{sortBy === "quantity" ? "Quantity" : sortBy === "availability" ? "Availability" : sortBy === "printing" ? "Printing" : "Card Name"}</Select.Trigger><Select.Content><Select.Item value="name">Card Name</Select.Item><Select.Item value="printing">Printing</Select.Item><Select.Item value="quantity">Quantity</Select.Item><Select.Item value="availability">Availability</Select.Item></Select.Content></Select.Root>
        <Button variant="outline" size="icon" class={`h-10! w-10! ${sortAscending ? "hover:bg-primary! hover:text-primary-foreground" : "bg-primary! text-primary-foreground hover:text-primary-foreground"}`} aria-label={sortAscending ? "Ascending" : "Descending"} onclick={() => sortAscending = !sortAscending}><Icon name={sortAscending ? "sort-asc" : "sort-desc"} size={15} /></Button>
        <CollectionFilters bind:selectedColors bind:selectedTypes bind:selectedSets sets={availableSets} />
        <Button variant="outline" size="icon" class="h-10! w-10!" aria-label="Quick Add" onclick={() => quickOpen = true}><Icon name="plus" size={18} /></Button>
      </div>
      <div class="overflow-x-auto"><Table.Root><Table.Header><Table.Row><Table.Head>Card</Table.Head><Table.Head>Printing</Table.Head><Table.Head>Quantity</Table.Head><Table.Head>Source</Table.Head><Table.Head></Table.Head></Table.Row></Table.Header><Table.Body>{#each visibleEntries as entry}<Table.Row><Table.Cell><strong class={entry.collection_quantity ? "text-foreground" : "text-muted-foreground"}>{entry.name}</strong><small class="block text-xs text-muted">{entry.collection_quantity ? `${entry.collection_quantity} available` : "missing"}</small></Table.Cell><Table.Cell>{entry.set_code} · {entry.collector_number}</Table.Cell><Table.Cell>{entry.quantity}</Table.Cell><Table.Cell>{entry.catalog_only ? "Catalog" : "Owned"}</Table.Cell><Table.Cell><Button variant="ghost" size="icon" class="size-7" aria-label={`Remove ${entry.name}`} onclick={() => remove(entry.id)}><Icon name="trash" size={14} /></Button></Table.Cell></Table.Row>{:else}<Table.Row><Table.Cell colspan={5} class="p-8 text-center text-muted">{loading ? "Loading…" : "This Box is empty."}</Table.Cell></Table.Row>{/each}</Table.Body></Table.Root></div>
    {:else}<p class="py-16 text-center text-muted">Create a Box to start organizing cards.</p>{/if}
    {#if error}<p class="mt-4 text-sm text-destructive">{error}</p>{/if}
  </section>
</div>

<Dialog.Root bind:open={quickOpen}><Dialog.Content class="max-w-xl overflow-hidden border-border bg-panel-raised p-0 text-foreground"><Command.Root shouldFilter={false}><Command.Input bind:value={addQuery} oninput={searchCatalog} autofocus class="h-12 w-full bg-transparent text-sm" placeholder="Add a card…" aria-label="Search cards to add" /><Command.List class="max-h-80 p-2"><Command.Empty class="p-5 text-center text-sm text-muted">{addQuery ? "No cards found." : "Search by card name, set, or collector number."}</Command.Empty><div class="mb-2 flex gap-1"><Button variant={source === "catalog" ? "secondary" : "ghost"} size="sm" onclick={() => { source = "catalog"; ownedResults = []; searchCatalog(); }}>Catalog</Button><Button variant={source === "owned" ? "secondary" : "ghost"} size="sm" onclick={() => { source = "owned"; results = []; searchCatalog(); }}>Owned</Button></div>{#each results as card}<Command.Item value={card.uuid} onSelect={() => add(card)} class={`flex justify-between rounded-md px-3 py-2 text-left ${card.collection_quantity ? "text-foreground" : "text-muted-foreground"}`}><span><strong class="block">{card.name}</strong><small>{card.set_code} · {card.collector_number}</small></span></Command.Item>{/each}{#each ownedResults as card}<Command.Item value={card.id} onSelect={() => addOwned(card)} class="rounded-md px-3 py-2 text-left"><strong class="block">{card.name}</strong><small class="text-muted">{card.set_code} · {card.collector_number} · {card.quantity} available</small></Command.Item>{/each}</Command.List></Command.Root></Dialog.Content></Dialog.Root>
