<script lang="ts">
  import type { Snippet } from "svelte";

  import { getServerPort } from "$lib/api/data";

  import ServerProvider from "./ServerProvider.svelte";

  type Props = {
    children?: Snippet;
  };

  const { children }: Props = $props();
</script>

{#await getServerPort()}
  <div class="skeleton-list">
    <div class="skeleton" style="width: 90%; height: 1.5rem;"></div>
    <div class="skeleton" style="width: 70%; height: 1rem;"></div>
    <div class="skeleton" style="width: 80%; height: 1rem;"></div>
  </div>
{:then port}
  <ServerProvider serverURL="http://127.0.0.1:{port}/" {children} />
{/await}
