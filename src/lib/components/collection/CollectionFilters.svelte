<script lang="ts">
 import * as Popover from "$lib/components/ui/popover";
 import { Button } from "$lib/components/ui/button";
 import { Checkbox } from "$lib/components/ui/checkbox";
 import Icon from "../ui/Icon.svelte";
 export let sets: string[] = [];
 export let selectedColors = new Set<string>();
 export let selectedTypes = new Set<string>();
 export let selectedSets = new Set<string>();
 const colors = ["White", "Blue", "Black", "Red", "Green", "Colorless"];
 const types = ["Creature", "Planeswalker", "Instant", "Enchantment", "Sorcery", "Land"];
 let colorOpen = true;
 let setOpen = true;
 let typeOpen = true;
 const toggle = (set: Set<string>, value: string, checked: boolean) => { const next = new Set(set); checked ? next.add(value) : next.delete(value); return next; };
 const isSelected = (group: string, item: string) => group === "Color" ? selectedColors.has(item) : group === "Set" ? selectedSets.has(item) : selectedTypes.has(item);
 $: active = selectedColors.size + selectedTypes.size + selectedSets.size;
</script>
<Popover.Root>
 <Popover.Trigger asChild>
  <Button variant="outline" size="icon" class="relative h-10! w-10!" aria-label="Filters" title="Filters" aria-haspopup="dialog"><Icon name="filter" size={17} />{#if active}<span class="absolute -right-1 -top-1 inline-flex size-4 items-center justify-center rounded-full bg-primary text-[10px] text-primary-foreground">{active}</span>{/if}</Button>
 </Popover.Trigger>
 <Popover.Content align="end" class="max-h-[min(34rem,calc(100vh-2rem))] w-72 overflow-y-auto">
  <div class="mb-3 flex items-center justify-between"><h3 class="font-semibold">Filters</h3>{#if active}<span class="text-xs text-muted">{active} active</span>{/if}</div>
  <section class="border-t border-border py-3 first:border-0 first:pt-0">
   <button type="button" class="mb-2 flex w-full items-center justify-between text-left text-xs font-bold uppercase tracking-wider text-muted" aria-expanded={colorOpen} aria-controls="filter-color-options" onclick={() => colorOpen = !colorOpen}><span>Color</span><Icon name="chevronDown" size={15} class={colorOpen ? "" : "-rotate-90"} /></button>
   {#if colorOpen}<div id="filter-color-options" class="grid grid-cols-2 gap-2">
    {#each colors as color}
     <label class="flex cursor-pointer items-center gap-2 text-sm"><Checkbox checked={selectedColors.has(color)} aria-label={`Filter by Color: ${color}`} onCheckedChange={(event) => selectedColors = toggle(selectedColors, color, event)} />{color}</label>
    {/each}
   </div>{/if}
  </section>
  <section class="border-t border-border py-3">
   <button type="button" class="mb-2 flex w-full items-center justify-between text-left text-xs font-bold uppercase tracking-wider text-muted" aria-expanded={setOpen} aria-controls="filter-set-options" onclick={() => setOpen = !setOpen}><span>Set</span><Icon name="chevronDown" size={15} class={setOpen ? "" : "-rotate-90"} /></button>
   {#if setOpen}<div id="filter-set-options" class="grid grid-cols-2 gap-2">
    {#each sets as set}
     <label class="flex cursor-pointer items-center gap-2 text-sm"><Checkbox checked={selectedSets.has(set)} aria-label={`Filter by Set: ${set}`} onCheckedChange={(event) => selectedSets = toggle(selectedSets, set, event)} />{set}</label>
    {/each}
   </div>{/if}
  </section>
  <section class="border-t border-border py-3">
   <button type="button" class="mb-2 flex w-full items-center justify-between text-left text-xs font-bold uppercase tracking-wider text-muted" aria-expanded={typeOpen} aria-controls="filter-type-options" onclick={() => typeOpen = !typeOpen}><span>Type</span><Icon name="chevronDown" size={15} class={typeOpen ? "" : "-rotate-90"} /></button>
   {#if typeOpen}<div id="filter-type-options" class="grid grid-cols-2 gap-2">
    {#each types as type}
     <label class="flex cursor-pointer items-center gap-2 text-sm"><Checkbox checked={selectedTypes.has(type)} aria-label={`Filter by Type: ${type}`} onCheckedChange={(event) => selectedTypes = toggle(selectedTypes, type, event)} />{type}</label>
    {/each}
   </div>{/if}
  </section>
 </Popover.Content>
</Popover.Root>
