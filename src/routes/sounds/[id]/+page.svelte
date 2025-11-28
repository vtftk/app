<script lang="ts">
  import { resolve } from "$app/paths";
  import { createSoundQuery } from "$lib/api/soundModel";
  import SoundForm from "$lib/sections/sounds/SoundForm.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";

  import type { PageProps } from "./$types";

  const { params }: PageProps = $props();

  const soundQuery = createSoundQuery(() => params.id);
</script>

{#if soundQuery.isLoading}
  <div class="skeleton-list">
    <div class="skeleton" style="width: 90%; height: 1.5rem;"></div>
    <div class="skeleton" style="width: 70%; height: 1rem;"></div>
    <div class="skeleton" style="width: 80%; height: 1rem;"></div>
  </div>
{:else if soundQuery.data}
  <SoundForm existing={soundQuery.data} />
{:else}
  {#snippet actions()}
    <a type="button" href={resolve("/sounds")}>Back</a>
  {/snippet}

  <PageLayoutList
    title="Sound Not Found"
    description="Unknown sound"
    {actions}
  />
{/if}

<style>
  .skeleton-list {
    padding: 1rem;
  }
</style>
