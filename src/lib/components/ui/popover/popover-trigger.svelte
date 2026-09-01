<script lang="ts">import { Popover as P } from "bits-ui"; let { children, ...props } = $props();</script>
<P.Trigger {...props}>{@render children?.()}</P.Trigger>
