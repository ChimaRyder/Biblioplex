<script lang="ts">
  import { DropdownMenu as Primitive } from "bits-ui";
  import { cn, type WithoutChild } from "$lib/utils.js";
  let { class: className, children, ...rest }: WithoutChild<Primitive.TriggerProps> = $props();
</script>
<Primitive.Trigger class={cn(className)} {...rest}>{@render children?.()}</Primitive.Trigger>
