<script lang="ts">
  import { resolve } from "$app/paths";
  import { createCommandQuery } from "$lib/api/commandModel";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import CommandForm from "$lib/sections/commands/CommandForm.svelte";

  import type { PageProps } from "./$types";

  const { params }: PageProps = $props();

  const commandQuery = createCommandQuery(() => params.id);
</script>

{#if commandQuery.isLoading}
  <div class="skeleton-list">
    <div class="skeleton" style="width: 90%; height: 1.5rem;"></div>
    <div class="skeleton" style="width: 70%; height: 1rem;"></div>
    <div class="skeleton" style="width: 80%; height: 1rem;"></div>
  </div>
{:else if commandQuery.data}
  <CommandForm existing={commandQuery.data} />
{:else}
  {#snippet actions()}
    <a type="button" href={resolve("/commands")}>Back</a>
  {/snippet}

  <PageLayoutList
    title="Command Not Found"
    description="Unknown command"
    {actions}
  />
{/if}

<style>
  .skeleton-list {
    padding: 1rem;
  }
</style>
