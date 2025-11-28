<script lang="ts">
  import { resolve } from "$app/paths";
  import { createEventQuery } from "$lib/api/eventModel";
  import EventForm from "$lib/sections/events/EventForm.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";

  import type { PageProps } from "./$types";

  const { params }: PageProps = $props();

  const eventQuery = $derived(createEventQuery(params.id));
</script>

{#if $eventQuery.isLoading}
  <div class="skeleton-list">
    <div class="skeleton" style="width: 90%; height: 1.5rem;"></div>
    <div class="skeleton" style="width: 70%; height: 1rem;"></div>
    <div class="skeleton" style="width: 80%; height: 1rem;"></div>
  </div>
{:else if $eventQuery.data}
  <EventForm existing={$eventQuery.data} />
{:else}
  {#snippet actions()}
    <LinkButton href={resolve("/events")}>Back</LinkButton>
  {/snippet}

  <PageLayoutList
    title="Event Not Found"
    description="Unknown event"
    {actions}
  />
{/if}

<style>
  .skeleton-list {
    padding: 1rem;
  }
</style>
