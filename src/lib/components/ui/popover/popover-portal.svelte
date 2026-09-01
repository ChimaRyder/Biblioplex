<script lang="ts">import { Popover as P } from "bits-ui"; let { children, ...props } = $props();</script>
<P.Portal {...props}>{@render children?.()}</P.Portal>
