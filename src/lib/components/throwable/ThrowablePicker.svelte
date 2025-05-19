<script lang="ts">
  import type { Item } from "$lib/api/types";

  import getBackendURL from "$lib/utils/url";
  import { filterNameSearch } from "$lib/utils/search";
  import { createItemsQuery } from "$lib/api/itemModel";
  import Dialog from "$lib/components/dialog/Dialog.svelte";
  import SearchInput from "$lib/components/form/SearchInput.svelte";
  import DialogCloseButton from "$lib/components/dialog/DialogCloseButton.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";

  import { getServerContext } from "../ServerProvider.svelte";

  type Props = {
    selected: string[];
    onChangeSelect: (selected: string[]) => void;
  };

  const { selected, onChangeSelect }: Props = $props();

  let search = $state("");

  const serverContext = getServerContext();
  const itemsQuery = createItemsQuery();

  const items = $derived($itemsQuery.data ?? []);
  const filteredItems = $derived(filterNameSearch(items, search));
  const selectedOptions = $derived(filterOptionsSelected(items, selected));

  function filterOptionsSelected(options: Item[], selected: string[]) {
    return options.filter((option) => selected.includes(option.id));
  }

  const onSelectItem = (item: Item) => {
    if (selected.includes(item.id)) {
      onChangeSelect(selected.filter((id) => id !== item.id));
    } else {
      onChangeSelect([...selected, item.id]);
    }
  };

  const onToggleAll = () => {
    if (items.length > 0 && selected.length === items.length) {
      onChangeSelect([]);
    } else {
      onChangeSelect(items.map((item) => item.id));
    }
  };
</script>

{#if $itemsQuery.isLoading}
  <div class="skeleton" style="width: 90%; height: 1.5rem; padding: 1rem"></div>
{/if}

<Dialog
  buttonLabel={{
    text:
      selectedOptions.length > 0
        ? `${selectedOptions.length} Items selected`
        : "Select Items",
  }}
>
  {#snippet title()}
    Select Items
  {/snippet}

  {#snippet description()}
    Choose which items will be thrown
  {/snippet}

  {#snippet children()}
    <div class="selection">
      <ControlledCheckbox
        checked={items.length > 0 && selected.length === items.length}
        onCheckedChange={onToggleAll}
      />

      <SearchInput bind:value={search} placeholder="Search" />
    </div>

    <div class="throwable-table-wrapper">
      <div class="items">
        {#each filteredItems as item (item.id)}
          <div class="item">
            <div class="item-column item-column--checkbox">
              <ControlledCheckbox
                checked={selected.includes(item.id)}
                onCheckedChange={() => onSelectItem(item)}
              />
            </div>

            <div class="item-column item-column--preview">
              <div class="throwable__image-wrapper">
                <img
                  class="throwable__image"
                  src={getBackendURL(serverContext, item.config.image.src)}
                  alt="Throwable"
                />
              </div>
            </div>

            <div class="item-column item-column--name">{item.name}</div>
          </div>
        {/each}
      </div>
    </div>
  {/snippet}

  {#snippet actions()}
    <DialogCloseButton buttonLabel={{ text: "Close" }} />
  {/snippet}
</Dialog>

{#if selectedOptions.length > 0}
  <div class="selected">
    <p class="selected__title">Selected Items</p>

    <div class="grid-wrapper">
      <div class="grid">
        {#each selectedOptions as option}
          <li class="grid-item">
            <div class="grid-item__image throwable__image-wrapper">
              <img
                class="throwable__image"
                src={getBackendURL(serverContext, option.config.image.src)}
                alt="Throwable"
              />
            </div>

            <p class="grid-item__name">{option.name}</p>
          </li>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .selected {
    margin-top: 0.5rem;
    display: flex;
    gap: 1rem;
    flex-flow: column;
    background-color: #333;
    padding: 1rem;
  }

  .selection {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding-left: 1rem;
    padding-right: 1rem;
    padding-bottom: 1rem;
  }

  .items {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }

  .selected__title {
    color: #fff;
    font-weight: bold;
  }

  .grid-wrapper {
    max-height: 12rem;
    overflow: auto;
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 0.5rem;
  }

  .grid-item {
    display: flex;
    gap: 1rem;
    align-items: center;
    width: 100%;
    overflow: hidden;
  }

  .grid-item__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .throwable__image {
    width: 2.5rem;
    height: 2.5rem;
    object-fit: contain;
    background-color: #333;
    border-radius: 2rem;
  }

  .throwable-table-wrapper {
    padding: 1rem;
    max-height: 300px;
    overflow-y: auto;
    width: 100%;
    min-height: 300px;
  }

  .item {
    display: flex;
    flex-flow: row;
    gap: 1rem;
    align-items: center;
  }
</style>
