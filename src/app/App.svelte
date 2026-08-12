<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let status = "Starting local workspace…";
  let backendReady = false;

  onMount(async () => {
    try {
      status = await invoke<string>("app_status");
      backendReady = true;
    } catch {
      status = "Web preview mode — Tauri backend is not connected.";
    }
  });
</script>

<svelte:head>
  <title>MTG Collection Manager</title>
</svelte:head>

<main class="shell">
  <header class="topbar">
    <div>
      <p class="eyebrow">LOCAL-FIRST · OFFLINE READY</p>
      <h1>MTG Collection Manager</h1>
    </div>
    <span class:ready={backendReady} class="status">
      <span class="status-dot"></span>
      {backendReady ? "Local workspace ready" : "Initializing"}
    </span>
  </header>

  <section class="hero">
    <div>
      <p class="eyebrow">FOUNDATION BUILD</p>
      <h2>Your collection, kept close.</h2>
      <p class="summary">
        A calm, account-free home for your cards, Boxes, Decks, and Tags.
        Core services are the next layer of the build.
      </p>
    </div>
    <div class="hero-mark" aria-hidden="true">✦</div>
  </section>

  <section class="workspace-grid" aria-label="Application areas">
    <article class="workspace-card active">
      <span class="card-index">01</span>
      <h3>Collection</h3>
      <p>Search and manage owned cards in compact offline mode.</p>
      <span class="card-state">Scaffolded</span>
    </article>
    <article class="workspace-card">
      <span class="card-index">02</span>
      <h3>Boxes</h3>
      <p>Organize physical locations and move cards safely.</p>
      <span class="card-state">Coming next</span>
    </article>
    <article class="workspace-card">
      <span class="card-index">03</span>
      <h3>Decks</h3>
      <p>Assemble owned Decks with quantity-aware allocation.</p>
      <span class="card-state">Coming next</span>
    </article>
    <article class="workspace-card">
      <span class="card-index">04</span>
      <h3>Catalog</h3>
      <p>Import a lightweight local projection of MTGJSON data.</p>
      <span class="card-state">Coming next</span>
    </article>
  </section>

  <footer>
    <span>Backend: {status}</span>
    <span>MTGJSON metadata · optional Scryfall images</span>
  </footer>
</main>
