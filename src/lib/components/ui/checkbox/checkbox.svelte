<script lang="ts">
  import { Checkbox as CheckboxPrimitive } from "bits-ui";
  import { Check } from "phosphor-svelte";
  import { cn } from "$lib/utils.js";

  let { class: className, ...restProps }: CheckboxPrimitive.RootProps = $props();
</script>

<CheckboxPrimitive.Root
  class={cn("peer size-4 shrink-0 rounded-sm border border-primary shadow focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground", className)}
  {...restProps}
>
  {#snippet children({ checked })}
    {#if checked}<Check class="size-3.5"/>{/if}
  {/snippet}
</CheckboxPrimitive.Root>
