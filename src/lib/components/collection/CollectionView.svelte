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

  type CardFace = { face_order: number; name: string; mana_cost?: string; card_type?: string; power?: string; toughness?: string; oracle_text?: string; image: { cached_path?: string; remote_url?: string; status: "cached" | "missing" | "stale" | "unavailable" } };
  type Card = { id: string; name: string; set_code: string; collector_number: string; mana_cost?: string; card_type?: string; power?: string; toughness?: string; quantity: number; language: string; foil: boolean; condition: string; notes?: string; rarity?: string; oracle_text?: string; faces: CardFace[] };
  type CatalogCard = { uuid: string; name: string; set_code: string; collector_number: string; rarity?: string };

  let cards: Card[] = [];
  let collectionQuery = "";
  let quickQuery = "";
  let quickResults: CatalogCard[] = [];
  let loading = true;
  let error = "";
  let quickAdding = "";
  let quickOpen = false;
  let sortBy = "name";
  let removing = "";
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
  $: activeFace = selectedCard?.faces?.[activeFaceIndex] ?? selectedCard?.faces?.[0];

  $: displayedCards = cards.sort((a, b) => sortBy === "quantity" ? b.quantity - a.quantity : a.name.localeCompare(b.name));
  function manaTokens(cost?: string) { return cost?.match(/\{[^}]+\}/g)?.map((token) => token.slice(1, -1).toLowerCase().replace("/", "")) ?? []; }
  async function loadCollection(query = collectionQuery) { loading = true; try { cards = await invoke<Card[]>("search_owned_cards", { request: { query } }); error = ""; } catch (err) { error = String(err); } finally { loading = false; } }
  function searchAsYouType() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => loadCollection(collectionQuery), 180);
  }
  async function searchQuickAdd(value = quickQuery) { quickQuery = value; if (!quickQuery.trim()) { quickResults = []; return; } try { quickResults = await invoke<CatalogCard[]>("catalog_search", { request: { query: quickQuery, limit: 8 } }); } catch (err) { error = String(err); } }
  async function quickAdd(card: CatalogCard) { quickAdding = card.uuid; try { await invoke("add_owned_catalog_card", { request: { printing_id: card.uuid, quantity: 1, language: "en", foil: false, condition: "near_mint", notes: null } }); quickQuery = ""; quickResults = []; quickOpen = false; await loadCollection(); } catch (err) { error = String(err); } finally { quickAdding = ""; } }
  async function removeCard(id: string) { if (removing !== id) { removing = id; return; } try { await invoke("remove_owned_card", { id }); await loadCollection(); } catch (err) { error = String(err); } finally { removing = ""; } }
  function openEdit(card: Card) { editingCard = card; editQuantity = card.quantity; editLanguage = card.language; editFoil = String(card.foil); editCondition = card.condition; editNotes = card.notes ?? ""; editOpen = true; }
  async function saveEdit() { if (!editingCard || editQuantity < 1) return; savingEdit = true; try { await invoke("update_owned_card", { request: { id: editingCard.id, quantity: editQuantity, language: editLanguage, foil: editFoil === "true", condition: editCondition, notes: editNotes || null } }); editOpen = false; await loadCollection(); } catch (err) { error = String(err); } finally { savingEdit = false; } }
  function openQuickAdd() { quickOpen = true; quickQuery = ""; quickResults = []; }
  function openViewer(card: Card) { selectedCard = card; activeFaceIndex = 0; imageFailed = false; viewerOpen = true; }
  function selectFace(index: number) { activeFaceIndex = index; imageFailed = false; }
  onMount(() => loadCollection(""));
</script>

<section class="relative mt-8 rounded-2xl border border-border bg-panel p-7 max-sm:mt-5 max-sm:p-5">
  <div class="mb-5 flex items-end justify-between gap-4"><div><p class="mb-2 text-[11px] font-bold tracking-[.16em] text-[#8b9bbd]">OWNED CARDS</p><h2 class="text-2xl tracking-tight">Your collection</h2></div><span class="text-xs text-muted">{displayedCards.length} records</span></div>
  <div class="mb-6 flex items-end gap-3 max-lg:flex-wrap">
    <div class="flex h-10 min-w-[260px] flex-1 items-center gap-2 rounded-md border border-[#52607d] bg-background px-3 transition focus-within:border-gold focus-within:ring-2 focus-within:ring-gold/20 lg:mr-12"><Icon name="search" size={17} /><Input class="h-9! border-0! bg-transparent! shadow-none! outline-none! ring-0! focus:border-transparent! focus:ring-0! focus-visible:border-transparent! focus-visible:ring-0!" bind:value={collectionQuery} placeholder="Search your collection…" aria-label="Search your collection" oninput={searchAsYouType} onkeydown={(event) => event.key === "Enter" && loadCollection()} /></div>
    <div class="ml-auto flex items-end gap-3 max-lg:ml-0">
      <label class="grid min-w-32 gap-2 text-[11px] font-bold uppercase tracking-wide text-[#8b9bbd]">Sort by<select class="h-10 appearance-none rounded-md border border-[#303d5d] bg-background px-3 text-sm text-foreground outline-none transition focus:border-gold focus:ring-2 focus:ring-gold/20" bind:value={sortBy} aria-label="Sort collection"><option value="name">Card name</option><option value="quantity">Quantity</option></select></label>
      <Button variant="outline" size="icon" class="h-10! w-10!" aria-label="Add a card" title="Add a card" onclick={openQuickAdd}><Icon name="plus" size={18} /></Button>
    </div>
  </div>

  <Dialog.Root bind:open={quickOpen}>
    <Dialog.Content class="max-w-xl overflow-hidden border-[#3a4663] bg-panel-raised p-0 text-foreground" showCloseButton={false}>
      {#snippet children()}
        <div class="flex items-center justify-between border-b border-border px-4 py-3"><div><Dialog.Title class="font-semibold">Add a card</Dialog.Title><Dialog.Description class="text-xs text-muted">Search the local catalog</Dialog.Description></div><Dialog.Close class="inline-flex size-8 items-center justify-center rounded-md text-muted transition hover:bg-[#202d48] hover:text-foreground" aria-label="Close add card command menu"><Icon name="x" size={16} /></Dialog.Close></div>
        <Command.Root class="bg-transparent text-foreground" shouldFilter={false}>
          <Command.Input bind:value={quickQuery} oninput={() => searchQuickAdd(quickQuery)} autofocus class="h-12 w-full bg-transparent text-sm outline-none placeholder:text-[#647394]" placeholder="Search card name, set, or collector number…" aria-label="Search catalog" />
          <Command.List class="max-h-80 overflow-y-auto p-2"><Command.Empty class="p-5 text-center text-sm text-muted">{quickQuery ? "No catalog matches. Import MTGJSON from Settings first." : "Start typing to find a card to add."}</Command.Empty><Command.Group>{#each quickResults as result (result.uuid)}<Command.Item value={`${result.name} ${result.set_code} ${result.collector_number} ${result.uuid}`} onSelect={() => quickAdd(result)} class="flex w-full cursor-pointer items-center justify-between rounded-md px-3 py-2 text-left outline-none data-[highlighted]:bg-[#202d48]" disabled={quickAdding === result.uuid}><span><strong class="block">{result.name}</strong><small class="text-xs text-[#8b9bbd]">{result.set_code} · {result.collector_number} · {result.rarity || "unknown"}</small></span>{#if quickAdding === result.uuid}<span class="text-xs text-[#f7d889]">Adding…</span>{/if}</Command.Item>{/each}</Command.Group></Command.List>
        </Command.Root>
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
            <div class="flex items-center justify-between gap-4 border-b border-border pb-4"><Dialog.Title class="min-w-0 break-words font-serif text-2xl leading-tight">{activeFace?.name}</Dialog.Title>{#if manaTokens(activeFace?.mana_cost).length}<span class="mana-cell flex shrink-0 items-center text-lg" aria-label={`Mana cost: ${activeFace?.mana_cost}`}>{#each manaTokens(activeFace?.mana_cost) as token}<i class="ms ms-cost ms-{token}" aria-hidden="true"></i>{/each}</span>{/if}</div>
            <div class="flex items-center justify-between gap-4"><p class="text-sm text-[#aab5ce]">{activeFace?.card_type || "Card"}</p>{#if activeFace?.card_type?.toLowerCase().includes("creature") && activeFace.power && activeFace.toughness}<span class="text-lg font-semibold text-foreground" aria-label={`Power ${activeFace.power}, toughness ${activeFace.toughness}`}>{activeFace.power}/{activeFace.toughness}</span>{/if}</div>
            {#if activeFace?.oracle_text}<div class="border-t border-border pt-4"><h3 class="mb-2 text-xs font-bold uppercase tracking-wide text-[#8b9bbd]">Card text</h3><p class="whitespace-pre-wrap text-sm leading-relaxed text-[#c4cce0]">{activeFace.oracle_text}</p></div>{/if}
            <div class="flex flex-wrap items-center gap-2 border-t border-border pt-4"><span class="rounded-full bg-[#202d48] px-2.5 py-1 text-xs font-semibold text-[#f7d889]">{selectedCard.set_code}</span><span class="text-xs text-muted">#{selectedCard.collector_number}</span>{#if selectedCard.rarity}<span class="text-xs capitalize text-muted">· {selectedCard.rarity}</span>{/if}</div>
            <div class="grid grid-cols-2 gap-3 border-t border-border pt-4 text-sm"><div><span class="block text-xs text-muted">Quantity</span><strong class="text-[#f7d889]">{selectedCard.quantity}</strong></div><div><span class="block text-xs text-muted">Condition</span><strong class="capitalize">{selectedCard.condition.replace("_", " ")}</strong></div><div><span class="block text-xs text-muted">Language</span><strong>{selectedCard.language.toUpperCase()}</strong></div><div><span class="block text-xs text-muted">Finish</span><strong>{selectedCard.foil ? "Foil" : "Non-foil"}</strong></div></div>
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
        <Dialog.Title>Edit {editingCard?.name}</Dialog.Title>
        <Dialog.Description>Update collection details. Card metadata cannot be changed here.</Dialog.Description>
      </Dialog.Header>
      <div class="grid gap-4 py-2">
        <label>Quantity<Input type="number" min="1" bind:value={editQuantity} /></label>
        <label>Language<Select.Root type="single" bind:value={editLanguage}>
          <Select.Trigger class="w-full">{editLanguage === "en" ? "English" : editLanguage === "ja" ? "Japanese" : editLanguage === "de" ? "German" : editLanguage === "fr" ? "French" : "Spanish"}</Select.Trigger>
          <Select.Content>
            {#each [{ value: "en", label: "English" }, { value: "ja", label: "Japanese" }, { value: "de", label: "German" }, { value: "fr", label: "French" }, { value: "es", label: "Spanish" }] as option}
            <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </label>
      <label>Finish<Select.Root type="single" bind:value={editFoil}><Select.Trigger class="w-full">{editFoil === "true" ? "Foil" : "Non-foil"}</Select.Trigger><Select.Content><Select.Item value="false" label="Non-foil">Non-foil</Select.Item><Select.Item value="true" label="Foil">Foil</Select.Item></Select.Content></Select.Root></label>
      <label>Condition<Select.Root type="single" bind:value={editCondition}><Select.Trigger class="w-full">{editCondition.replace("_", " ")}</Select.Trigger><Select.Content>{#each [{ value: "near_mint", label: "Near mint" }, { value: "lightly_played", label: "Lightly played" }, { value: "moderately_played", label: "Moderately played" }, { value: "heavily_played", label: "Heavily played" }, { value: "damaged", label: "Damaged" }] as option}<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>{/each}</Select.Content></Select.Root></label>
      <label>Notes<Textarea bind:value={editNotes} rows="3" /></label>
      </div>
      <Dialog.Footer>
        <Dialog.Close>Cancel</Dialog.Close>
        <Button onclick={saveEdit} disabled={savingEdit || editQuantity < 1}>{savingEdit ? "Saving…" : "Save changes"}</Button>
      </Dialog.Footer>
      {/snippet}
    </Dialog.Content>
  </Dialog.Root>
  {#if error}<p class="mt-4 text-sm text-red-300" role="alert">{error}</p>{/if}
  {#if loading}<p class="text-sm text-muted">Loading local collection…</p>{:else if displayedCards.length === 0}<div class="grid justify-items-center gap-2 py-12 text-center text-muted"><Icon name="grid" size={28} /><p class="font-semibold text-foreground">{collectionQuery ? "No owned cards match your search." : "Your collection is empty."}</p><small>{collectionQuery ? "Try a different search." : "Use the + button to add your first card."}</small></div>{:else}<div class="overflow-x-auto"><table class="w-full text-left text-sm"><thead class="text-[11px] uppercase tracking-wide text-muted"><tr class="border-b border-border"><th class="p-3">Qty</th><th class="p-3">Card name</th><th class="p-3">Mana cost</th><th class="p-3">Type</th><th class="p-3">Printing</th><th class="p-3"><span class="sr-only">Actions</span></th></tr></thead><tbody>{#each displayedCards as card (card.id)}<tr class="cursor-pointer border-b border-border text-[#aab5ce] transition hover:bg-[#18233d]" tabindex="0" role="button" aria-label={`View details for ${card.name}`} onclick={() => openViewer(card)} onkeydown={(event) => (event.key === "Enter" || event.key === " ") && openViewer(card)}><td class="p-3 font-bold text-[#f7d889]">{card.quantity}</td><td class="p-3"><strong class="text-foreground">{card.name}</strong>{#if card.foil}<span class="ml-2 text-[10px] font-bold text-[#d6ae58]">FOIL</span>{/if}<small class="mt-1 block text-[11px] text-[#71809f] md:hidden">{card.condition} · {card.language}</small></td><td class="mana-cell p-3">{#if manaTokens(card.mana_cost).length}{#each manaTokens(card.mana_cost) as token}<i class="ms ms-cost ms-{token}" aria-label={token}></i>{/each}{:else}—{/if}</td><td class="p-3">{card.card_type || "—"}</td><td class="p-3">{card.set_code} · {card.collector_number || "—"}</td><td class="p-3"><Button variant="ghost" size="icon" class="size-8" aria-label={`Edit `} title={`Edit `} onclick={(event) => { event.stopPropagation(); console.trace("test"); openEdit(card); }}><Icon name="pencil" size={15} /></Button><Button variant="destructive" size="sm" aria-label={removing === card.id ? `Confirm removal of ${card.name}` : `Remove ${card.name}`} onclick={(event) => { event.stopPropagation(); removeCard(card.id); }}><Icon name={removing === card.id ? "x" : "trash"} size={14} />{removing === card.id ? "Confirm" : ""}</Button></td></tr>{/each}</tbody></table></div>{/if}
</section>
