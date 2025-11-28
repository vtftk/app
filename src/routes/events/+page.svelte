<script lang="ts">
  import type { VEvent } from "$lib/api/types";

  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import ExportIcon from "~icons/solar/export-bold";
  import { toastErrorMessage } from "$lib/utils/error";
  import { filterNameSearch } from "$lib/utils/search";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import Button from "$lib/components/input/Button.svelte";
  import EventItem from "$lib/sections/events/EventItem.svelte";
  import { createSelection } from "$lib/utils/selection.svelte";
  import ImportEvents from "$lib/components/ImportEvents.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import SearchInput from "$lib/components/form/SearchInput.svelte";
  import VirtualOrderableGrid from "$lib/components/VirtualOrderableGrid.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import {
    deleteEvents,
    exportEvents,
    updateEventOrder,
    createEventsQuery,
  } from "$lib/api/eventModel";

  const eventsQuery = createEventsQuery();

  let search = $state("");

  const events = $derived($eventsQuery.data ?? []);
  const selection = createSelection(() => events);
  const filteredEvents = $derived(filterNameSearch(events, search));

  async function onBulkDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete the selected events?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = deleteEvents(selection.take());

    toast.promise(deletePromise, {
      loading: "Deleting events...",
      success: "Deleted events",
      error: toastErrorMessage("Failed to delete events"),
    });
  }

  async function onBulkExport() {
    const exportPromise = exportEvents(selection.take());

    toast.promise(exportPromise, {
      loading: "Exporting events...",
      success: "Exported events",
      error: toastErrorMessage("Failed to export events"),
    });
  }
</script>

<PageLayoutList
  title="Events"
  description="Setup specific triggers for events, such as throwing when a specific redeem is redeemed"
>
  {#snippet actions()}
    <ImportEvents />

    <LinkButton href={resolve("/events/create")}>Create</LinkButton>
  {/snippet}

  {#snippet beforeContent()}
    <div class="selection">
      <ControlledCheckbox
        checked={selection.isAll()}
        onCheckedChange={() => selection.toggleAll()}
      />

      <div class="search-wrapper">
        <SearchInput bind:value={search} placeholder="Search..." />
      </div>

      {#if !selection.isEmpty()}
        <div class="selection__count">
          {selection.total()} Selected
        </div>
      {/if}

      <div class="selection__gap"></div>

      <div class="selection__actions">
        <Button onclick={onBulkExport} disabled={selection.isEmpty()}>
          <ExportIcon /> Export
        </Button>

        <Button onclick={onBulkDelete} disabled={selection.isEmpty()}>
          <DeleteIcon /> Delete
        </Button>
      </div>
    </div>
  {/snippet}

  <VirtualOrderableGrid
    items={filteredEvents}
    onUpdateOrder={updateEventOrder}
    disableOrdering={search.length > 0}
    itemHeight={110}
  >
    <!-- Snippet for rendering items within the grid -->
    {#snippet item(event: VEvent)}
      <EventItem
        config={event}
        selected={selection.includes(event.id)}
        onToggleSelected={() => selection.toggle(event.id)}
      />
    {/snippet}
  </VirtualOrderableGrid>
</PageLayoutList>

<style>
  .selection {
    display: flex;
    align-items: center;
    gap: 1rem;
    height: 3rem;
    flex-shrink: 0;
  }

  .selection__gap {
    flex: auto;
  }

  .selection__actions {
    display: flex;
    gap: 1rem;
  }

  .search-wrapper {
    display: flex;
    flex: auto;
    flex-shrink: 1;
    flex-grow: 0;
    max-width: 20rem;
  }
</style>
