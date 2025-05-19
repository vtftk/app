<script lang="ts">
  // Apply global styling
  import "$lib/styles/global.css";
  // Font family for code editor
  import "@fontsource/jetbrains-mono";
  import { fly } from "svelte/transition";
  import { page, navigating } from "$app/state";
  import { queryClient } from "$lib/api/client";
  import Sidebar from "$lib/components/nav/Sidebar.svelte";
  import AppToaster from "$lib/components/AppToaster.svelte";
  import { QueryClientProvider } from "@tanstack/svelte-query";
  import AppDataProvider from "$lib/sections/AppDataProvider.svelte";
  import UpdateNotification from "$lib/components/update/UpdateNotification.svelte";
  import InternalServerProvider from "$lib/components/InternalServerProvider.svelte";
  import GlobalConfirmDialog from "$lib/components/dialog/GlobalConfirmDialog.svelte";
</script>

<!-- Toast popup provider -->
<AppToaster />

<GlobalConfirmDialog />

<!-- Global query client context -->
<QueryClientProvider client={queryClient}>
  <InternalServerProvider>
    <!-- App data loader and context provider -->
    <AppDataProvider>
      <main class="main">
        <Sidebar />

        {#key page.url}
          <div class="content" in:fly={{ y: -100, duration: 250 }}>
            {#if navigating.to}
              <div class="skeleton-list">
                <div class="skeleton" style="width: 90%; height: 1.5rem;"></div>
                <div class="skeleton" style="width: 70%; height: 1rem;"></div>
                <div class="skeleton" style="width: 80%; height: 1rem;"></div>
              </div>
            {:else}
              <slot />
            {/if}
          </div>
        {/key}
      </main>

      <UpdateNotification />
    </AppDataProvider>
  </InternalServerProvider>
</QueryClientProvider>

<style>
  .main {
    display: flex;
    width: 100%;
    height: 100vh;
    overflow: hidden;
  }

  .content {
    display: flex;
    flex-flow: column;
    position: relative;
    flex: auto;
    height: 100%;
    overflow: hidden;
  }

  .skeleton-list {
    padding: 1rem;
  }
</style>
