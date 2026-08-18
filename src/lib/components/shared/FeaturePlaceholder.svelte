<script lang="ts">
  import Icon from "../ui/Icon.svelte";
  export let kind: "boxes" | "decks";
  $: isBoxes = kind === "boxes";
</script>

<section class="mx-auto grid max-w-xl place-items-center py-[18vh] text-center" aria-labelledby="placeholder-title">
  <div class="mb-5 text-primary"><Icon name={isBoxes ? "box" : "deck"} size={48} /></div>
  <p class="mb-2 text-[11px] font-bold tracking-[.16em] text-muted-foreground">{isBoxes ? "PHYSICAL ORGANIZATION" : "ASSEMBLED DECKS"}</p>
  <h2 id="placeholder-title" class="mb-3 font-serif text-4xl tracking-tight">{isBoxes ? "Organize your shelves." : "Build your decks."}</h2>
  <p class="mb-6 leading-relaxed text-muted-foreground">{isBoxes ? "Boxes will let you create storage locations, see their contents, and move owned cards safely." : "Decks will let you assemble owned cards into Mainboard, Sideboard, and Commander sections."}</p>
  <span class="rounded-full border border-border px-3 py-2 text-[11px] text-muted">Available in the next organization milestone</span>
</section>
