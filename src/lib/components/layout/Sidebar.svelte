<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "../ui/Icon.svelte";

  export let page: "collection" | "boxes" | "decks" | "settings";
  export let navigate: (page: "collection" | "boxes" | "decks" | "settings") => void;
  const items = [
    ["collection", "Collection", "grid"],
    ["boxes", "Boxes", "box"],
    ["decks", "Decks", "deck"],
    ["settings", "Settings", "settings"],
  ] as const;
</script>

<aside class="fixed inset-y-0 left-0 z-20 flex w-60 flex-col border-r border-border bg-[#0e1527] px-4 py-7 max-md:w-20 max-md:px-2">
  <div class="mb-14 flex items-center gap-3 px-2 text-foreground max-md:justify-center"><div class="grid size-9 place-items-center rounded-lg border border-[#8b7137] bg-[#221e2d] text-xl text-[#f7c873]" aria-hidden="true">✦</div><div class="max-md:hidden"><strong class="block font-serif text-xl tracking-tight">Biblioplex</strong><small class="mt-1 block text-[10px] text-[#71809f]">MTG collection manager</small></div></div>
  <nav class="grid gap-1" aria-label="Main navigation">
    {#each items as [id, label, icon]}
      <Button variant={page === id ? "outline" : "ghost"} class="w-full justify-start gap-3 border-0 px-3 py-3 text-left text-sm max-md:justify-center max-md:px-0 max-md:text-[0px]" aria-current={page === id ? "page" : undefined} onclick={() => navigate(id)}>
        <span class="text-base"><Icon name={icon} /></span>{label}
      </Button>
    {/each}
  </nav>
  <div class="mt-auto flex flex-wrap items-center gap-2 border-t border-border px-3 pt-4 text-xs text-[#8b9bbd] max-md:justify-center max-md:px-0"><span class="size-2 rounded-full bg-emerald-400"></span><span class="max-md:hidden">Local workspace</span><small class="w-full pl-4 text-[#62718f] max-md:hidden">Offline ready</small></div>
</aside>
