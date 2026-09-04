<script lang="ts">
  import { DropdownMenu as Primitive } from "bits-ui";
  let { open = $bindable(false), ...rest }: Primitive.RootProps = $props();
</script>
<Primitive.Root bind:open {...rest} />
