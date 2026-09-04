<script lang="ts">
  import { DropdownMenu as Primitive } from "bits-ui";
  import { cn, type WithoutChild } from "$lib/utils.js";
  let { class: className, children, ...rest }: WithoutChild<Primitive.ItemProps> = $props();
</script>
<Primitive.Item class={cn("relative flex w-full cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none focus:bg-accent data-[highlighted]:bg-accent data-[disabled]:pointer-events-none data-[disabled]:opacity-50", className)} {...rest}>{@render children?.()}</Primitive.Item>
