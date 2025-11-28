<script lang="ts">
  import type { Command } from "$lib/api/types";

  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import ExportIcon from "~icons/solar/export-bold";
  import { toastErrorMessage } from "$lib/utils/error";
  import { filterNameSearch } from "$lib/utils/search";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import Button from "$lib/components/input/Button.svelte";
  import { createSelection } from "$lib/utils/selection.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import SearchInput from "$lib/components/form/SearchInput.svelte";
  import ImportCommands from "$lib/components/ImportCommands.svelte";
  import CommandItem from "$lib/sections/commands/CommandItem.svelte";
  import VirtualOrderableGrid from "$lib/components/VirtualOrderableGrid.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import {
    exportCommands,
    updateCommandOrder,
    bulkDeleteCommands,
    createCommandsQuery,
  } from "$lib/api/commandModel";

  const commandsQuery = createCommandsQuery();

  let search = $state("");

  const commands = $derived($commandsQuery.data ?? []);
  const selection = createSelection(() => commands);
  const filteredCommands = $derived(filterNameSearch(commands, search));

  async function onBulkDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete the selected commands?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = bulkDeleteCommands(selection.take());

    toast.promise(deletePromise, {
      loading: "Deleting commands...",
      success: "Deleted commands",
      error: toastErrorMessage("Failed to delete commands"),
    });
  }

  async function onBulkExport() {
    const exportPromise = exportCommands(selection.take());

    toast.promise(exportPromise, {
      loading: "Exporting commands...",
      success: "Exported commands",
      error: toastErrorMessage("Failed to export commands"),
    });
  }
</script>

<PageLayoutList title="Commands" description="Custom twitch commands">
  {#snippet actions()}
    <ImportCommands />

    <LinkButton href={resolve("/commands/create")}>Create</LinkButton>
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
    items={filteredCommands}
    onUpdateOrder={updateCommandOrder}
    disableOrdering={search.length > 0}
  >
    {#snippet item(item: Command)}
      <CommandItem
        config={item}
        selected={selection.includes(item.id)}
        onToggleSelected={() => selection.toggle(item.id)}
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
