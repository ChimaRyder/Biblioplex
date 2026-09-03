<script lang="ts">
  import { DropdownMenu as Primitive } from "bits-ui";
  import { cn, type WithoutChild } from "$lib/utils.js";
  let { class: className, children, sideOffset = 4, ...rest }: WithoutChild<Primitive.ContentProps> = $props();
</script>
<Primitive.Portal><Primitive.Content sideOffset={sideOffset} class={cn("z-50 min-w-36 overflow-hidden rounded-md border border-border bg-panel-raised p-1 text-foreground shadow-lg", className)} {...rest}>{@render children?.()}</Primitive.Content></Primitive.Portal>
