<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "../lib/components/layout/Sidebar.svelte";
  import PageHeader from "../lib/components/layout/PageHeader.svelte";
  import CollectionView from "../lib/components/collection/CollectionView.svelte";
  import FeaturePlaceholder from "../lib/components/shared/FeaturePlaceholder.svelte";
  import CatalogSettings from "../lib/components/settings/CatalogSettings.svelte";
  import BoxesView from "../lib/components/boxes/BoxesView.svelte";
  import { Toaster } from "../lib/components/ui/sonner";

  type Page = "collection" | "boxes" | "decks" | "settings";
  let page: Page = "collection";
  const pageTitles: Record<Page, string> = { collection: "Collection", boxes: "Boxes", decks: "Decks", settings: "Settings" };
  function navigate(next: Page) { page = next; }
  onMount(() => { document.documentElement.dataset.appReady = "true"; });
</script>

<svelte:head><title>Biblioplex · {pageTitles[page]}</title></svelte:head>
<div class="min-h-screen bg-background">
  <Sidebar {page} {navigate} />
  <main class="ml-60 min-h-screen px-12 py-10 max-md:ml-20 max-md:px-6 max-sm:px-4">
    {#if page !== "collection"}<PageHeader title={pageTitles[page]} />{/if}
    {#if page === "collection"}<CollectionView />{:else if page === "boxes"}<BoxesView />{:else if page === "settings"}<CatalogSettings />{:else}<FeaturePlaceholder kind={page} />{/if}
  </main>
</div>
<Toaster position="bottom-right" richColors />
