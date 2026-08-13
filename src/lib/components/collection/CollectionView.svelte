<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import * as Command from "$lib/components/ui/command";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import Icon from "../ui/Icon.svelte";
  import { Input } from "$lib/components/ui/input";

  type Card = { id: string; name: string; set_code: string; collector_number: string; mana_cost?: string; card_type?: string; quantity: number; language: string; foil: boolean; condition: string; notes?: string };
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
  function openQuickAdd() { quickOpen = true; quickQuery = ""; quickResults = []; }
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

  {#if error}<p class="mt-4 text-sm text-red-300" role="alert">{error}</p>{/if}
  {#if loading}<p class="text-sm text-muted">Loading local collection…</p>{:else if displayedCards.length === 0}<div class="grid justify-items-center gap-2 py-12 text-center text-muted"><Icon name="grid" size={28} /><p class="font-semibold text-foreground">{collectionQuery ? "No owned cards match your search." : "Your collection is empty."}</p><small>{collectionQuery ? "Try a different search." : "Use the + button to add your first card."}</small></div>{:else}<div class="overflow-x-auto"><table class="w-full text-left text-sm"><thead class="text-[11px] uppercase tracking-wide text-muted"><tr class="border-b border-border"><th class="p-3">Qty</th><th class="p-3">Card name</th><th class="p-3">Mana cost</th><th class="p-3">Type</th><th class="p-3">Printing</th><th class="p-3"><span class="sr-only">Actions</span></th></tr></thead><tbody>{#each displayedCards as card (card.id)}<tr class="border-b border-border text-[#aab5ce]"><td class="p-3 font-bold text-[#f7d889]">{card.quantity}</td><td class="p-3"><strong class="text-foreground">{card.name}</strong>{#if card.foil}<span class="ml-2 text-[10px] font-bold text-[#d6ae58]">FOIL</span>{/if}<small class="mt-1 block text-[11px] text-[#71809f] md:hidden">{card.condition} · {card.language}</small></td><td class="mana-cell p-3">{#if manaTokens(card.mana_cost).length}{#each manaTokens(card.mana_cost) as token}<i class="ms ms-cost ms-{token}" aria-label={token}></i>{/each}{:else}—{/if}</td><td class="p-3">{card.card_type || "—"}</td><td class="p-3">{card.set_code} · {card.collector_number || "—"}</td><td class="p-3"><Button variant="destructive" size="sm" aria-label={removing === card.id ? `Confirm removal of ${card.name}` : `Remove ${card.name}`} onclick={() => removeCard(card.id)}><Icon name={removing === card.id ? "x" : "trash"} size={14} />{removing === card.id ? "Confirm" : "Remove"}</Button></td></tr>{/each}</tbody></table></div>{/if}
</section>
