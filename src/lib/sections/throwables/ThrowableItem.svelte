<script lang="ts">
  import type { Item } from "$lib/api/types";

  import { toast } from "svelte-sonner";
  import getBackendURL from "$lib/utils/url";
  import { toastErrorMessage } from "$lib/utils/error";
  import SettingsIcon from "~icons/solar/settings-bold";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import { deleteItemMutation } from "$lib/api/itemModel";
  import BallsIcon from "~icons/solar/balls-bold-duotone";
  import Button from "$lib/components/input/Button.svelte";
  import SolarMenuDotsBold from "~icons/solar/menu-dots-bold";
  import BallIcon from "~icons/solar/basketball-bold-duotone";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import PopoverCloseButton from "$lib/components/popover/PopoverCloseButton.svelte";

  type Props = {
    config: Item;

    selected?: boolean;
    onToggleSelected?: VoidFunction;

    testingEnabled: boolean;
    onTestThrow: (itemIds: string[]) => void;
    onTestBarrage: (itemIds: string[]) => void;
  };

  const {
    config,
    selected,
    onToggleSelected,
    testingEnabled,
    onTestThrow,
    onTestBarrage,
  }: Props = $props();

  const deleteItem = deleteItemMutation();

  async function onDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete this item?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = $deleteItem.mutateAsync(config.id);

    toast.promise(deletePromise, {
      loading: "Deleting item...",
      success: "Deleted item",
      error: toastErrorMessage("Failed to delete item"),
    });
  }
</script>

{#snippet popoverContent()}
  <PopoverCloseButton
    disabled={!testingEnabled}
    onclick={() => onTestThrow([config.id])}
  >
    <BallIcon /> Test One
  </PopoverCloseButton>

  <PopoverCloseButton
    disabled={!testingEnabled}
    onclick={() => onTestBarrage([config.id])}
  >
    <BallsIcon /> Test Barrage
  </PopoverCloseButton>

  <LinkButton href="/throwables/{config.id}">
    <SettingsIcon /> View
  </LinkButton>

  <Button onclick={onDelete} disabled={$deleteItem.isPending}>
    <DeleteIcon /> Delete
  </Button>
{/snippet}

<div class="item">
  {#if onToggleSelected}
    <ControlledCheckbox
      checked={selected ?? false}
      onCheckedChange={onToggleSelected}
    />
  {/if}
  <div class="item__content">
    <div class="item__image-wrapper">
      <img
        class="item__image"
        class:item__image--pixelate={config.config.image.pixelate}
        src={getBackendURL(config.config.image.src)}
        alt="Throwable"
        loading="lazy"
      />
    </div>
  </div>

  <div class="item__text">
    <a href="/throwables/{config.id}" class="item__name">{config.name}</a>
  </div>

  <PopoverButton
    content={popoverContent}
    contentProps={{ align: "start", side: "left" }}
  >
    <SolarMenuDotsBold />
  </PopoverButton>
</div>

<style>
  .item {
    background-color: #1a1a1a;
    border: 1px solid #2f2f2f;
    border-radius: 5px;

    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;

    padding: 0.5rem;
    overflow: hidden;
  }

  .item__content {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .item__image {
    width: 42px;
    height: 42px;
    object-fit: contain;
    background-color: #333;
    border-radius: 2rem;
    flex-shrink: 0;
  }

  .item__image--pixelate {
    image-rendering: pixelated;
  }

  .item__text {
    display: flex;
    flex: auto;
    align-items: center;
    overflow: hidden;
  }

  .item__name {
    flex: 1;
    color: #fff;
    font-weight: bold;

    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;

    text-decoration: none;
  }

  .item__name:hover {
    text-decoration: underline;
  }
</style>
