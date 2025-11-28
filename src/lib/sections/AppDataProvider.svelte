<script lang="ts">
  import type { AppData, RuntimeAppData } from "$lib/api/types";

  import { setContext, type Snippet } from "svelte";
  import {
    APP_CONTEXT_KEY,
    createAppDataQuery,
    createRuntimeAppDataQuery,
  } from "$lib/api/runtimeAppData";

  type Props = {
    children: Snippet;
  };

  const { children }: Props = $props();

  const runtimeAppData = createRuntimeAppDataQuery();
  const runtimeAppDataStore: RuntimeAppData | undefined = $derived(
    runtimeAppData.data,
  );

  const appData = createAppDataQuery();
  const appDataStore: AppData | undefined = $derived(appData.data);

  setContext(APP_CONTEXT_KEY, {
    // Values within the context are guaranteed to be defined when the context is used
    get appData() {
      return appDataStore!;
    },

    get runtimeAppData() {
      return runtimeAppDataStore!;
    },
  });
</script>

{#if runtimeAppData.isLoading || appData.isLoading}
  <div class="skeleton-list">
    <div class="skeleton" style="width: 90%; height: 1.5rem;"></div>
    <div class="skeleton" style="width: 70%; height: 1rem;"></div>
    <div class="skeleton" style="width: 80%; height: 1rem;"></div>
  </div>
{:else if appDataStore !== undefined || runtimeAppDataStore !== undefined}
  {@render children()}
{/if}

<style>
  .skeleton-list {
    padding: 1rem;
  }
</style>
