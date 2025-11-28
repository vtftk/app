<script lang="ts">
  import type { Item, Sound } from "$lib/api/types";

  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import { toastErrorMessage } from "$lib/utils/error";
  import { filterNameSearch } from "$lib/utils/search";
  import SettingsIcon from "~icons/solar/settings-bold";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import BallsIcon from "~icons/solar/balls-bold-duotone";
  import { getAppContext } from "$lib/api/runtimeAppData";
  import Button from "$lib/components/input/Button.svelte";
  import BallIcon from "~icons/solar/basketball-bold-duotone";
  import { createSelection } from "$lib/utils/selection.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import { testThrow, testThrowBarrage } from "$lib/api/throwables";
  import SearchInput from "$lib/components/form/SearchInput.svelte";
  import SoundPicker from "$lib/components/sounds/SoundPicker.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import ThrowableItem from "$lib/sections/throwables/ThrowableItem.svelte";
  import VirtualOrderableGrid from "$lib/components/VirtualOrderableGrid.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import PopoverCloseButton from "$lib/components/popover/PopoverCloseButton.svelte";
  import BulkThrowableImport from "$lib/components/throwable/BulkThrowableImport.svelte";
  import {
    updateItemOrder,
    bulkDeleteItems,
    createItemsQuery,
    bulkAppendItemSounds,
  } from "$lib/api/itemModel";

  const appContext = getAppContext();
  const runtimeAppData = $derived(appContext.runtimeAppData);

  const itemsQuery = createItemsQuery();

  let search = $state("");

  const items = $derived(itemsQuery.data ?? []);
  const selection = createSelection(() => items);
  const filteredItems = $derived(filterNameSearch(items, search));

  // Testing is only available when an overlay and vtube studio is connected
  const testingEnabled = $derived(
    runtimeAppData.active_overlay_count > 0 &&
      runtimeAppData.vtube_studio_connected,
  );

  async function onBulkDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete the selected throwables?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = bulkDeleteItems(selection.take());

    toast.promise(deletePromise, {
      loading: "Deleting items...",
      success: "Deleted items",
      error: toastErrorMessage("Failed to delete items"),
    });
  }

  async function onBulkAddSounds(sounds: Sound[]) {
    const confirm = await confirmDialog({
      title: "Confirm Add Sounds",
      description:
        "Are you sure you want to add the selected impact sounds to the selected throwables?",
    });

    if (!confirm) {
      return;
    }

    const impactSoundIds = sounds.map((sound) => sound.id);

    const addPromise = bulkAppendItemSounds(
      selection.selection,
      impactSoundIds,
    );

    toast.promise(addPromise, {
      loading: "Adding impact sounds...",
      success: "Added impact sounds",
      error: toastErrorMessage("Failed to add impact sounds"),
    });
  }

  /**
   * Performs a test throw of the provided items
   *
   * @param itemIds The IDs of the items to throw
   */
  function onTestThrow(itemIds: string[]) {
    const throwPromise = testThrow(itemIds, 1);

    toast.promise(throwPromise, {
      loading: "Sending throw...",
      success: "Threw item",
      error: toastErrorMessage("Failed to throw item"),
    });
  }

  function onTestBarrage(itemIds: string[]) {
    const throwPromise = testThrowBarrage(itemIds, 20, 2, 100);

    toast.promise(throwPromise, {
      loading: "Sending barrage...",
      success: "Threw barrage",
      error: toastErrorMessage("Failed to throw barrage"),
    });
  }
</script>

<PageLayoutList title="Throwables" description="Items that can be thrown.">
  <!-- Actions in the titlebar -->
  {#snippet actions()}
    <PopoverButton>
      Create

      <!-- Content for the "Test" button popover -->
      {#snippet content()}
        <LinkButton href={resolve("/throwables/create")}>
          Create Throwable
        </LinkButton>
        <BulkThrowableImport />
      {/snippet}
    </PopoverButton>
  {/snippet}

  <!-- Section before the content -->
  {#snippet beforeContent()}
    <div class="selection">
      <ControlledCheckbox
        checked={selection.isAll()}
        onCheckedChange={selection.toggleAll}
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
        <PopoverButton disabled={!testingEnabled || selection.isEmpty()}>
          <BallIcon /> Test

          {#snippet content()}
            <PopoverCloseButton
              onclick={() => onTestThrow(selection.selection)}
            >
              <BallIcon /> Test One
            </PopoverCloseButton>

            <PopoverCloseButton
              onclick={() => onTestBarrage(selection.selection)}
            >
              <BallsIcon /> Test Barrage
            </PopoverCloseButton>
          {/snippet}
        </PopoverButton>

        <SoundPicker
          disabled={selection.isEmpty()}
          description="Choose which impact sounds you'd like to add the the selected throwables."
          selected={[]}
          onChangeSelected={onBulkAddSounds}
        >
          {#snippet buttonContent()}
            <SettingsIcon /> Add Impact Sounds
          {/snippet}
        </SoundPicker>

        <Button onclick={onBulkDelete} disabled={selection.isEmpty()}>
          <DeleteIcon /> Delete
        </Button>
      </div>
    </div>
  {/snippet}

  <VirtualOrderableGrid
    items={filteredItems}
    onUpdateOrder={updateItemOrder}
    disableOrdering={search.length > 0}
  >
    {#snippet item(item: Item)}
      <ThrowableItem
        config={item}
        selected={selection.includes(item.id)}
        onToggleSelected={() => selection.toggle(item.id)}
        {testingEnabled}
        {onTestThrow}
        {onTestBarrage}
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
    align-items: center;
  }

  .search-wrapper {
    display: flex;
    flex: auto;
    flex-shrink: 1;
    flex-grow: 0;
    max-width: 20rem;
  }
</style>
