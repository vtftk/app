<script lang="ts">
  import type { Sound } from "$lib/api/types";

  import { toast } from "svelte-sonner";
  import { toastErrorMessage } from "$lib/utils/error";
  import { filterNameSearch } from "$lib/utils/search";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import Button from "$lib/components/input/Button.svelte";
  import SoundItem from "$lib/sections/sounds/SoundItem.svelte";
  import { createSelection } from "$lib/utils/selection.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import SearchInput from "$lib/components/form/SearchInput.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import BulkSoundImport from "$lib/components/sounds/BulkSoundImport.svelte";
  import VirtualOrderableGrid from "$lib/components/VirtualOrderableGrid.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import {
    deleteSounds,
    updateSoundOrder,
    createSoundsQuery,
  } from "$lib/api/soundModel";

  const soundsQuery = createSoundsQuery();

  let search = $state("");

  const sounds = $derived($soundsQuery.data ?? []);
  const selection = createSelection(() => sounds);
  const filteredSounds: Sound[] = $derived(filterNameSearch(sounds, search));

  async function onBulkDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete the selected sounds?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = deleteSounds(selection.take());

    toast.promise(deletePromise, {
      loading: "Deleting sounds...",
      success: "Deleted sounds",
      error: toastErrorMessage("Failed to delete sounds"),
    });
  }
</script>

<PageLayoutList
  title="Sounds"
  description="Create sounds that can be used for events or use as impact sounds"
>
  {#snippet actions()}
    <PopoverButton>
      Create

      {#snippet content()}
        <LinkButton href="/sounds/create">Create Sound</LinkButton>
        <BulkSoundImport />
      {/snippet}
    </PopoverButton>
  {/snippet}

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
        <Button onclick={onBulkDelete} disabled={selection.isEmpty()}>
          <DeleteIcon /> Delete
        </Button>
      </div>
    </div>
  {/snippet}

  <VirtualOrderableGrid
    items={filteredSounds}
    onUpdateOrder={updateSoundOrder}
    disableOrdering={search.length > 0}
  >
    {#snippet item(sound: Sound)}
      <SoundItem
        config={sound}
        selected={selection.includes(sound.id)}
        onToggleSelected={() => selection.toggle(sound.id)}
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
