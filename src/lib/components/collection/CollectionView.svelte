<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import * as Command from "$lib/components/ui/command";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import Icon from "../ui/Icon.svelte";
  import { Input } from "$lib/components/ui/input";
  import Textarea from "$lib/components/ui/textarea/textarea.svelte";
  import * as Select from "$lib/components/ui/select";
  import * as Table from "$lib/components/ui/table";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { checkImageProvider, type ConnectionState } from "$lib/commands/connection";

  type CardFace = { face_order: number; name: string; mana_cost?: string; card_type?: string; power?: string; toughness?: string; oracle_text?: string; image: { cached_path?: string; remote_url?: string; status: "cached" | "missing" | "stale" | "unavailable" } };
  type Card = { id: string; name: string; set_code: string; collector_number: string; mana_cost?: string; card_type?: string; power?: string; toughness?: string; quantity: number; language: string; foil: boolean; condition: string; notes?: string; rarity?: string; oracle_text?: string; faces: CardFace[] };
  type CatalogCard = { uuid: string; name: string; set_code: string; collector_number: string; rarity?: string };
  type DuplicateCard = { id: string; quantity: number; language: string; foil: boolean; condition: string; notes?: string };

  let cards: Card[] = [];
  let collectionQuery = "";
  let quickQuery = "";
  let quickResults: CatalogCard[] = [];
  let loading = true;
  let error = "";
  let quickAdding = "";
  let duplicateCard: { catalog: CatalogCard; rows: DuplicateCard[] } | null = null;
  let quickOpen = false;
  let sortBy = "name";
  let viewMode: "list" | "grid" = "list";
  let imageLoading = new Set<string>();
  let imageFailedCards = new Set<string>();
  let adjusting = "";
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  let selectedCard: Card | null = null;
  let viewerOpen = false;
  let imageFailed = false;
  let editOpen = false;
  let editingCard: Card | null = null;
  let editQuantity = 1;
  let editLanguage = "en";
  let editFoil = "false";
  let editCondition = "near_mint";
  let editNotes = "";
  let savingEdit = false;
  let activeFaceIndex = 0;
  let connectionState: ConnectionState = "unknown";
  let previewCard: Card | null = null;
  let previewPosition = { top: 0, left: 0 };
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
  let previewImageFailed = false;
  let selectedCardIds = new Set<string>();
  let bulkRemoving = false;
  $: activeFace = selectedCard?.faces?.[activeFaceIndex] ?? selectedCard?.faces?.[0];
  $: previewFace = previewCard?.faces?.[0];

  $: displayedCards = cards.sort((a, b) => sortBy === "quantity" ? b.quantity - a.quantity : a.name.localeCompare(b.name));
  $: totalCardQuantity = displayedCards.reduce((total, card) => total + card.quantity, 0);
  $: if (viewMode === "grid") clearPreview();
  function manaTokens(cost?: string) { return cost?.match(/\{[^}]+\}/g)?.map((token) => token.slice(1, -1).toLowerCase().replace("/", "")) ?? []; }
  async function loadCollection(query = collectionQuery) { loading = true; try { cards = await invoke<Card[]>("search_owned_cards", { request: { query } }); selectedCardIds = new Set([...selectedCardIds].filter((id) => cards.some((card) => card.id === id))); error = ""; } catch (err) { error = String(err); } finally { loading = false; } }
  function searchAsYouType() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => loadCollection(collectionQuery), 180);
  }
  async function searchQuickAdd(value = quickQuery) { quickQuery = value; if (!quickQuery.trim()) { quickResults = []; return; } try { quickResults = await invoke<CatalogCard[]>("catalog_search", { request: { query: quickQuery, limit: 8 } }); } catch (err) { error = String(err); } }
  async function quickAdd(card: CatalogCard) {
    quickAdding = card.uuid;
    try {
      const rows = await invoke<DuplicateCard[]>("find_owned_catalog_cards", { printingId: card.uuid });
      if (rows.length) { duplicateCard = { catalog: card, rows }; return; }
      await confirmQuickAdd(card);
    } catch (err) { error = String(err); } finally { quickAdding = ""; }
  }
  async function confirmQuickAdd(card: CatalogCard) {
    await invoke("add_owned_catalog_card", { request: { printing_id: card.uuid, quantity: 1, language: "en", foil: false, condition: "near_mint", notes: null } });
    duplicateCard = null; quickQuery = ""; quickResults = []; quickOpen = false; await loadCollection();
  }
  function cancelDuplicateWarning() { duplicateCard = null; quickAdding = ""; }
  async function adjustQuantity(card: Card, delta: 1 | -1) {
    if (adjusting) return;
    adjusting = card.id;
    try {
      if (delta === -1 && card.quantity === 1) {
        await invoke("remove_owned_card", { id: card.id });
      } else {
        await invoke("update_owned_card", { request: {
          id: card.id,
          quantity: card.quantity + delta,
          language: card.language,
          foil: card.foil,
          condition: card.condition,
          notes: card.notes || null
        } });
      }
      await loadCollection();
    } catch (err) { error = String(err); } finally { adjusting = ""; }
  }
  function openEdit(card: Card) { editingCard = card; editQuantity = card.quantity; editLanguage = card.language; editFoil = String(card.foil); editCondition = card.condition; editNotes = card.notes ?? ""; editOpen = true; }
  async function saveEdit() { if (!editingCard || editQuantity < 1) return; savingEdit = true; try { await invoke("update_owned_card", { request: { id: editingCard.id, quantity: editQuantity, language: editLanguage, foil: editFoil === "true", condition: editCondition, notes: editNotes || null } }); editOpen = false; await loadCollection(); } catch (err) { error = String(err); } finally { savingEdit = false; } }
  function openQuickAdd() { quickOpen = true; quickQuery = ""; quickResults = []; }
  function openViewer(card: Card) { selectedCard = card; activeFaceIndex = 0; imageFailed = false; viewerOpen = true; }
  function selectFace(index: number) { activeFaceIndex = index; imageFailed = false; }
  function imageSource(card: Card) { const image = card.faces?.[0]?.image; return image?.cached_path || (connectionState === "stable" ? image?.remote_url : undefined); }
  function toggleCardSelection(id: string) { const next = new Set(selectedCardIds); if (next.has(id)) next.delete(id); else next.add(id); selectedCardIds = next; }
  function clearSelection() { selectedCardIds = new Set(); }
  async function removeSelectedCards() { if (!selectedCardIds.size || bulkRemoving) return; bulkRemoving = true; try { await invoke("remove_owned_cards", { ids: [...selectedCardIds] }); clearSelection(); await loadCollection(); } catch (err) { error = String(err); } finally { bulkRemoving = false; } }
  function enterGridView() { if (connectionState !== "stable" || viewMode === "grid") return; clearSelection(); viewMode = "grid"; imageLoading = new Set(displayedCards.filter((card) => imageSource(card)).map((card) => card.id)); imageFailedCards = new Set(); }
  function markImageLoading(id: string) { imageLoading = new Set(imageLoading).add(id); }
  function markImageLoaded(id: string) { const next = new Set(imageLoading); next.delete(id); imageLoading = next; }
  function markImageFailed(id: string) { markImageLoaded(id); imageFailedCards = new Set(imageFailedCards).add(id); }
  function clearPreview() { if (previewTimer) clearTimeout(previewTimer); previewTimer = undefined; previewCard = null; previewImageFailed = false; }
  function schedulePreview(card: Card, event: PointerEvent, row?: HTMLElement) {
    clearPreview();
    if (connectionState !== "stable") return;
    const rect = (row ?? event.currentTarget as HTMLElement).getBoundingClientRect();
    previewPosition = { top: Math.max(12, Math.min(window.innerHeight - 420, rect.top)), left: Math.min(window.innerWidth - 280, rect.right + 12) };
    previewTimer = setTimeout(() => { if (connectionState === "stable") previewCard = card; }, 700);
  }
  async function refreshConnection() { connectionState = "checking"; connectionState = await checkImageProvider(); if (connectionState !== "stable") clearPreview(); }
  onMount(() => {
    loadCollection(""); refreshConnection();
    const onNetworkChange = () => refreshConnection();
    const onPointerOver = (event: PointerEvent) => { if (viewMode !== "list") return; const row = (event.target as HTMLElement).closest("tbody tr") as HTMLElement | null; if (!row || connectionState !== "stable") return; const rows = [...row.parentElement!.children]; const card = displayedCards[rows.indexOf(row)]; if (card) schedulePreview(card, event, row); };
    const onPointerOut = (event: PointerEvent) => { const row = (event.target as HTMLElement).closest("tbody tr"); if (row && !(event.relatedTarget as Node | null)?.parentElement?.closest?.("tbody tr")) clearPreview(); };
    document.addEventListener("pointerover", onPointerOver); document.addEventListener("pointerout", onPointerOut);
    window.addEventListener("online", onNetworkChange); window.addEventListener("offline", onNetworkChange);
    return () => { clearPreview(); document.removeEventListener("pointerover", onPointerOver); document.removeEventListener("pointerout", onPointerOut); window.removeEventListener("online", onNetworkChange); window.removeEventListener("offline", onNetworkChange); };
  });
</script>

<div class="mb-8 flex items-end gap-3 max-lg:flex-wrap">
  <div class="flex h-10 min-w-[260px] flex-1 items-center gap-2 rounded-md border border-[#52607d] bg-background px-3 transition focus-within:border-gold focus-within:ring-2 focus-within:ring-gold/20 lg:mr-12"><Icon name="search" size={17} /><Input class="h-9! border-0! bg-transparent! shadow-none! outline-none! ring-0! focus:border-transparent! focus:ring-0! focus-visible:border-transparent! focus-visible:ring-0!" bind:value={collectionQuery} placeholder="Search your collection…" aria-label="Search your collection" oninput={searchAsYouType} onkeydown={(event : KeyboardEvent)=> event.key === "Enter" && loadCollection()} /></div>
  <div class="ml-auto flex items-end gap-3 max-lg:ml-0">
    <div class="flex h-10 items-center rounded-md border border-[#303d5d] bg-background p-1" aria-label="Collection view">
      <Button variant={viewMode === "list" ? "secondary" : "ghost"} size="icon" class="size-8" aria-label="List View" title="List View" aria-pressed={viewMode === "list"} onclick={() => viewMode = "list"}><Icon name="list" size={17} /></Button>
      <Button variant={viewMode === "grid" ? "secondary" : "ghost"} size="icon" class="size-8" aria-label={connectionState === "stable" ? "Card Grid View" : "Card Grid View requires a stable connection"} title={connectionState === "stable" ? "Card Grid View" : "Card Grid requires a stable connection"} aria-pressed={viewMode === "grid"} disabled={connectionState !== "stable"} onclick={enterGridView}><Icon name="grid" size={17} /></Button>
    </div>
    <select class="h-10 min-w-32 appearance-none rounded-md border border-[#303d5d] bg-background px-3 text-sm text-foreground outline-none transition focus:border-gold focus:ring-2 focus:ring-gold/20" bind:value={sortBy} aria-label="Sort collection"><option value="name">Card Name</option><option value="quantity">Quantity</option></select>
    <Button variant="outline" size="icon" class="h-10! w-10!" aria-label="Add a card" title="Add a card" onclick={openQuickAdd}><Icon name="plus" size={18} /></Button>
  </div>
</div>

<section class="relative rounded-2xl border border-border bg-panel p-7 max-sm:p-5">
  <div class="mb-5 flex items-center gap-3"><h2 class="text-2xl tracking-tight">All Cards</h2><span class="inline-flex min-w-7 items-center justify-center rounded-full bg-accent px-2 py-0.5 text-xs font-semibold text-foreground" aria-label={`${totalCardQuantity} cards`}>{totalCardQuantity}</span></div>
  {#if selectedCardIds.size > 0}
    <div class="mb-4 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-[#806c3e] bg-[#2a2418] px-4 py-3" role="status" aria-live="polite">
      <span class="text-sm font-medium text-[#f7d889]">{selectedCardIds.size} {selectedCardIds.size === 1 ? "card" : "cards"} selected</span>
      <div class="flex gap-2"><Button variant="destructive" size="sm" disabled={bulkRemoving} onclick={removeSelectedCards} title="Remove Cards"><Icon name="trash" size={16} /></Button><Button variant="outline" size="sm" disabled={bulkRemoving} onclick={clearSelection} title="Cancel"><Icon name="x" size={16} /></Button></div>
    </div>
  {/if}

  <Dialog.Root bind:open={quickOpen}>
    <Dialog.Content class="max-w-xl overflow-hidden border-[#3a4663] bg-panel-raised p-0 text-foreground" showCloseButton={false}>
      {#snippet children()}
        {#if duplicateCard}
          <div class="grid gap-5 p-6">
            <div><Dialog.Title class="text-xl">Card already in collection</Dialog.Title><Dialog.Description class="mt-2 text-sm text-muted"><span class="font-bold">{duplicateCard.catalog.name} ({duplicateCard.catalog.set_code} · {duplicateCard.catalog.collector_number})</span>  already exists in your collection. Adding this card will create a separate row.</Dialog.Description></div>
            <div class="grid gap-2">{#each duplicateCard.rows as row}<div class="rounded-md border border-border bg-background px-3 py-2 text-sm"><strong>{row.quantity}×</strong><span class="ml-3">{row.language.toUpperCase()} · {row.foil ? "Foil" : "Non-foil"} · {row.condition.replace("_", " ")}</span></div>{/each}</div>
            <div class="flex justify-end gap-2"><Button variant="outline" onclick={cancelDuplicateWarning}>Cancel</Button><Button onclick={() => confirmQuickAdd(duplicateCard!.catalog)}>Add anyway</Button></div>
          </div>
        {:else}
          <div class="flex items-center justify-between border-b border-border px-4 py-3"><div><Dialog.Title class="font-semibold">Add a card</Dialog.Title><Dialog.Description class="text-xs text-muted">Search the local catalog</Dialog.Description></div><Dialog.Close class="inline-flex size-8 items-center justify-center rounded-md text-muted transition hover:bg-red-400/10 hover:text-red-300" aria-label="Close add card command menu"><Icon name="x" size={16} /></Dialog.Close></div>
        <Command.Root class="bg-transparent text-foreground" shouldFilter={false}>
          <Command.Input bind:value={quickQuery} oninput={() => searchQuickAdd(quickQuery)} autofocus class="h-12 w-full bg-transparent text-sm outline-none placeholder:text-[#647394]" placeholder="Search card name, set, or collector number…" aria-label="Search catalog" />
          <Command.List class="max-h-80 overflow-y-auto p-2"><Command.Empty class="p-5 text-center text-sm text-muted">{quickQuery ? "No catalog matches. Import MTGJSON from Settings first." : "Start typing to find a card to add."}</Command.Empty><Command.Group>{#each quickResults as result (result.uuid)}<Command.Item value={`${result.name} ${result.set_code} ${result.collector_number} ${result.uuid}`} onSelect={() => quickAdd(result)} class="flex w-full cursor-pointer items-center justify-between rounded-md px-3 py-2 text-left outline-none data-[highlighted]:bg-[#202d48]" disabled={quickAdding === result.uuid}><span><strong class="block">{result.name}</strong><small class="text-xs text-[#8b9bbd]">{result.set_code} · {result.collector_number} · {result.rarity || "unknown"}</small></span>{#if quickAdding === result.uuid}<span class="text-xs text-[#f7d889]">Adding…</span>{/if}</Command.Item>{/each}</Command.Group></Command.List>
        </Command.Root>
        {/if}
      {/snippet}
    </Dialog.Content>
  </Dialog.Root>

  <Dialog.Root bind:open={viewerOpen}>
    <Dialog.Content class="fixed inset-y-0 right-0 left-auto z-50 flex h-full w-full max-w-md translate-x-0 translate-y-0 flex-col gap-0 overflow-y-auto rounded-none border-l border-[#3a4663] bg-panel-raised p-0 text-foreground shadow-2xl data-open:animate-in data-open:slide-in-from-right data-closed:animate-out data-closed:slide-out-to-right" showCloseButton={false}>
      {#snippet children()}
        {#if selectedCard}
          <div class="flex items-center justify-between border-b border-border px-5 py-3"><Dialog.Title class="text-[10px] font-bold tracking-[.16em] text-[#8b9bbd]">CARD DETAILS</Dialog.Title><Dialog.Close class="inline-flex size-6 items-center justify-center rounded-md text-red-400 transition hover:bg-red-400/10 hover:text-red-300" aria-label="Close card details"><Icon name="x" size={13} /></Dialog.Close></div>
          <div class="grid gap-6 p-5">
            {#if (activeFace?.image.cached_path || activeFace?.image.remote_url) && !imageFailed}
              <div class="overflow-hidden rounded-xl border border-border bg-background"><img class="mx-auto block max-h-[520px] w-full object-contain" src={activeFace.image.cached_path || activeFace.image.remote_url} alt={`${activeFace.name} card art`} loading="lazy" onerror={() => (imageFailed = true)} /></div>
            {:else}<div class="grid min-h-40 place-items-center rounded-xl border border-dashed border-border bg-background p-6 text-center text-sm text-muted">{activeFace?.image.status === "stale" ? "Cached image is stale." : "Image unavailable offline."}<br />Local metadata is still available.</div>{/if}
            {#if selectedCard.faces.length > 1}<div class="flex justify-center"><Button variant="outline" size="sm" aria-label={`Flip card to ${activeFaceIndex === 0 ? "back" : "front"}`} onclick={() => selectFace(activeFaceIndex === 0 ? 1 : 0)}>↔ {activeFaceIndex === 0 ? "Show back" : "Show front"}</Button></div>{/if}
            <div class="flex items-center justify-between gap-4 border-b border-border pb-4"><Dialog.Title class="min-w-0 break-words font-serif text-2xl leading-tight">{activeFace?.name}</Dialog.Title>{#if manaTokens(activeFace?.mana_cost).length}<span class="mana-cell flex shrink-0 items-center text-lg" aria-label={`Mana cost: ${activeFace?.mana_cost}`}>{#each manaTokens(activeFace?.mana_cost) as token}<i class="ms ms-cost ms-{token}" aria-label={token}></i>{/each}</span>{/if}</div>
            <div class="flex items-center justify-between gap-4"><p class="text-sm text-[#aab5ce]">{activeFace?.card_type || "Card"}</p>{#if activeFace?.card_type?.toLowerCase().includes("creature") && activeFace.power && activeFace.toughness}<span class="text-lg font-semibold text-foreground" aria-label={`Power ${activeFace.power}, toughness ${activeFace.toughness}`}>{activeFace.power}/{activeFace.toughness}</span>{/if}</div>
            {#if activeFace?.oracle_text}<div class="border-t border-border pt-4"><h3 class="mb-2 text-xs font-bold uppercase tracking-wide text-[#8b9bbd]">Card text</h3><p class="whitespace-pre-wrap text-sm leading-relaxed text-[#c4cce0]">{activeFace.oracle_text}</p></div>{/if}
            <div class="flex flex-wrap items-center gap-2 border-t border-border pt-4"><span class="rounded-full bg-[#202d48] px-2.5 py-1 text-xs font-semibold text-[#f7d889]">{selectedCard.set_code}</span><span class="text-xs text-muted">#{selectedCard.collector_number}</span>{#if selectedCard.rarity}<span class="text-xs capitalize text-muted">· {selectedCard.rarity}</span>{/if}</div>
            <div class="grid grid-cols-2 gap-3 border-t border-border pt-4 text-sm"><div><span class="block text-xs text-muted">Quantity</span><strong class="text-[#f7d889]">{selectedCard.quantity}</strong></div><div><span class="block text-xs text-muted">Condition</span><strong class="capitalize">{selectedCard.condition.replace("_", " ")}</strong></div><div><span class="block text-xs text-muted">Language</span><strong>{selectedCard.language.toUpperCase()}</strong></div><div><span class="block text-xs text-muted">Finish</span><strong>{selectedCard.foil ? "Foil" : "Non-Foil"}</strong></div></div>
            {#if selectedCard.notes}<div class="border-t border-border pt-4"><h3 class="mb-2 text-xs font-bold uppercase tracking-wide text-[#8b9bbd]">Notes</h3><p class="text-sm leading-relaxed text-[#c4cce0]">{selectedCard.notes}</p></div>{/if}
          </div>
        {/if}
      {/snippet}
    </Dialog.Content>
  </Dialog.Root>

  <Dialog.Root bind:open={editOpen}>
    <Dialog.Content class="border-[#3a4663] bg-panel-raised text-foreground sm:max-w-lg" showCloseButton={false}>
      {#snippet children()}
      <Dialog.Header>
        <Dialog.Title>Edit <span class="italic">{editingCard?.name}</span></Dialog.Title>
      </Dialog.Header>
      <div class="grid gap-4 py-2">
        <label><div class="pb-1 font-medium">Quantity</div><Input type="number" min="1" bind:value={editQuantity} /></label>
        <label><div class="pb-1 font-medium">Language</div><Select.Root type="single" bind:value={editLanguage}>
          <Select.Trigger class="w-full">{editLanguage === "en" ? "English" : editLanguage === "ja" ? "Japanese" : editLanguage === "de" ? "German" : editLanguage === "fr" ? "French" : "Spanish"}</Select.Trigger>
          <Select.Content>
            {#each [{ value: "en", label: "English" }, { value: "ja", label: "Japanese" }, { value: "de", label: "German" }, { value: "fr", label: "French" }, { value: "es", label: "Spanish" }] as option}
            <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </label>
      <label><div class="pb-1 font-medium">Finish</div><Select.Root type="single" bind:value={editFoil}><Select.Trigger class="w-full">{editFoil === "true" ? "Foil" : "Non-Foil"}</Select.Trigger><Select.Content><Select.Item value="false" label="Non-foil">Non-foil</Select.Item><Select.Item value="true" label="Foil">Foil</Select.Item></Select.Content></Select.Root></label>
      <label><div class="pb-1 font-medium">Condition</div><Select.Root type="single" bind:value={editCondition}><Select.Trigger class="w-full">{editCondition.replace("_", " ").replace(/(^\w|\s\w)/g, m => m.toUpperCase())}</Select.Trigger><Select.Content>{#each [{ value: "near_mint", label: "Near Mint" }, { value: "lightly_played", label: "Lightly Played" }, { value: "moderately_played", label: "Moderately Played" }, { value: "heavily_played", label: "Heavily Played" }, { value: "Damaged", label: "Damaged" }] as option}<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>{/each}</Select.Content></Select.Root></label>
      <label><div class="pb-1 font-medium">Notes</div><Textarea bind:value={editNotes} rows="3" /></label>
      </div>
      <Dialog.Footer class="border-border bg-panel">
        <Dialog.Close class="inline-flex h-8 items-center justify-center rounded-lg border border-border bg-background px-2.5 text-sm font-medium text-foreground transition-colors outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50">Cancel</Dialog.Close>
        <Button onclick={saveEdit} disabled={savingEdit || editQuantity < 1}>{savingEdit ? "Saving…" : "Save changes"}</Button>
      </Dialog.Footer>
      {/snippet}
    </Dialog.Content>
  </Dialog.Root>
  {#if previewCard && previewFace}<div class="pointer-events-none fixed z-40 w-64 rounded-xl border border-[#52607d] bg-panel-raised p-2 shadow-2xl" style={`top:${previewPosition.top}px;left:${previewPosition.left}px`} aria-hidden="true">{#if (previewFace.image.cached_path || (connectionState === "stable" && previewFace.image.remote_url)) && !previewImageFailed}<img class="block max-h-[360px] w-full rounded-lg object-contain" src={previewFace.image.cached_path || previewFace.image.remote_url} alt="" loading="eager" onerror={() => { previewImageFailed = true; connectionState = "unavailable"; clearPreview(); }} />{:else}<div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-border bg-background p-4 text-center text-xs text-muted">Image unavailable.</div>{/if}<p class="truncate px-1 pt-2 text-xs font-semibold text-foreground">{previewCard.name}</p><p class="px-1 pb-1 text-[11px] text-muted">{previewCard.set_code} · {previewCard.collector_number}</p></div>{/if}
  {#if error}<p class="mt-4 text-sm text-destructive" role="alert">{error}</p>{/if}
  {#if loading}
    <p class="text-sm text-muted">Loading local collection…</p>
  {:else if displayedCards.length === 0}
    <div class="grid justify-items-center gap-2 py-12 text-center text-muted">
      <Icon name="grid" size={28} />
      <p class="font-semibold text-foreground">
        {collectionQuery ? "No owned cards match your search." : "Your collection is empty."}
      </p>
      <small>{collectionQuery ? "Try a different search." : "Use the + button to add your first card."}</small>
    </div>
  {:else if viewMode === "list"}
    <div class="overflow-x-auto">
      <Table.Root>
        <Table.Header>
          <Table.Row>
            <Table.Head><span class="sr-only">Select</span></Table.Head>
            <Table.Head><span class="sr-only">Add/Remove</span></Table.Head>
            <Table.Head>Card Name</Table.Head>
            <Table.Head>Mana Cost</Table.Head>
            <Table.Head>Type</Table.Head>
            <Table.Head>Printing</Table.Head>
            <Table.Head><span class="sr-only">Actions</span></Table.Head>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each displayedCards as card (card.id)}
            <Table.Row
              class="group cursor-pointer text-[#aab5ce]"
              tabindex="0"
              role="button"
              aria-label={`View details for ${card.name}`}
              onclick={() => openViewer(card)}
              onkeydown={(event: KeyboardEvent) => (event.key === "Enter" || event.key === " ") && openViewer(card)}
            >
              <Table.Cell class="w-8">
                <Checkbox checked={selectedCardIds.has(card.id)} aria-label={`Select ${card.name}`} class={`${selectedCardIds.size === 0 ? "opacity-0 group-hover:opacity-100" : "opacity-100"}`} onclick={(event: MouseEvent) => event.stopPropagation()} onCheckedChange={() => toggleCardSelection(card.id)} />
              </Table.Cell>
              <Table.Cell>
                <div class="flex items-center gap-3">
                  <Button variant="outline" size="icon" class="size-7 hover:!border-none hover:!bg-primary hover:!text-primary-foreground" aria-label={`Add Card`} title={`Add Card`} disabled={adjusting === card.id} onclick={(event: MouseEvent) => { event.stopPropagation(); adjustQuantity(card, 1); }}>
                    <Icon name="plus" size={14} />
                  </Button>
                  <div class="font-bold text-[#f7d889]">{card.quantity} x</div>
                  <Button variant="outline" size="icon" class="size-7 hover:!border-none hover:!bg-primary hover:!text-primary-foreground" aria-label={`Remove Card`} title={`Remove Card`} disabled={adjusting === card.id} onclick={(event: MouseEvent) => { event.stopPropagation(); adjustQuantity(card, -1); }}>
                    <Icon name="minus" size={14} />
                  </Button>
                </div>
              </Table.Cell>
              <Table.Cell>
                <strong class="text-foreground">{card.name}</strong>
                {#if card.foil}<span class="ml-2 text-[10px] font-bold text-[#d6ae58]">FOIL</span>{/if}
                <small class="mt-1 block text-[11px] text-[#71809f] md:hidden">{card.condition} · {card.language}</small>
              </Table.Cell>
              <Table.Cell class="mana-cell">
                {#if manaTokens(card.mana_cost).length}
                  {#each manaTokens(card.mana_cost) as token}<i class="ms ms-cost ms-{token}" aria-label={token}></i>{/each}
                {:else}—{/if}
              </Table.Cell>
              <Table.Cell>{card.card_type || "—"}</Table.Cell>
              <Table.Cell>{card.set_code} · {card.collector_number || "—"}</Table.Cell>
              <Table.Cell>
                <Button variant="ghost" size="icon" class="size-8" aria-label={`Edit ${card.name}`} title={`Edit ${card.name}`} onclick={(event: MouseEvent) => { event.stopPropagation(); openEdit(card); }}>
                  <Icon name="pencil" size={15} />
                </Button>
              </Table.Cell>
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>
  {:else}
    <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
      {#each displayedCards as card (card.id)}
        {@const face = card.faces?.[0]}
        {@const src = imageSource(card)}
        <article class="overflow-hidden rounded-xl border border-border bg-background shadow-sm transition-transform duration-200 ease-out hover:z-10 hover:scale-[1.03] hover:shadow-lg">
          <button class="cursor-pointer block w-full text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-label={`View details for ${card.name}`} onclick={() => openViewer(card)}>
            <div class="relative aspect-[5/7] overflow-hidden bg-muted">
              {#if src && !imageFailedCards.has(card.id)}
                {#if imageLoading.has(card.id)}<Skeleton class="absolute inset-2 rounded-lg" />{/if}
                <img class:opacity-0={imageLoading.has(card.id)} class="h-full w-full object-cover transition-opacity" src={src} alt={`${card.name} card art`} loading="lazy" onload={() => markImageLoaded(card.id)} onerror={() => markImageFailed(card.id)} />
              {:else}
                <div class="flex h-full items-center justify-center p-4 text-center text-xs text-muted">{face?.image.status === "stale" ? "Cached image is stale." : "Image unavailable."}</div>
              {/if}
            </div>
            <!-- <div class="grid gap-1 p-3">
              <strong class="truncate text-sm text-foreground">{card.name}</strong>
              <span class="text-xs text-muted">{card.quantity}x · {card.set_code} · #{card.collector_number}</span>
              <span class="truncate text-xs text-muted">{card.card_type || "Card"}{#if card.foil} · Foil{/if}</span>
            </div> -->
          </button>
        </article>
      {/each}
    </div>
  {/if}
</section>
