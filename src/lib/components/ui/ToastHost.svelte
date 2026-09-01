<script lang="ts">import { toast } from "$lib/toast";</script>
{#if $toast}<div class="fixed bottom-6 right-6 z-[100] max-w-sm rounded-lg border border-border bg-panel-raised px-4 py-3 text-sm text-foreground shadow-2xl" role="status" aria-live="polite">{$toast.message}</div>{/if}
