<script lang="ts">
  import type { Command } from "$lib/api/types";

  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import { toastErrorMessage } from "$lib/utils/error";
  import SettingsIcon from "~icons/solar/settings-bold";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import Button from "$lib/components/input/Button.svelte";
  import SolarMenuDotsBold from "~icons/solar/menu-dots-bold";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import { deleteCommand, updateCommand } from "$lib/api/commandModel";
  import EnabledSwitch from "$lib/components/input/EnabledSwitch.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";

  type Props = {
    config: Command;

    selected: boolean;
    onToggleSelected: VoidFunction;
  };

  const { config, selected, onToggleSelected }: Props = $props();

  async function onDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete this command?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = deleteCommand(config.id);

    toast.promise(deletePromise, {
      loading: "Deleting command...",
      success: "Deleted command",
      error: toastErrorMessage("Failed to delete command"),
    });
  }

  async function onChangeDisabled(value: boolean) {
    const updatePromise = updateCommand({
      commandId: config.id,
      update: {
        enabled: value,
      },
    });

    toast.promise(updatePromise, {
      loading: value ? "Enabling..." : "Disabling...",
      success: value ? "Enabled " + config.name : "Disabled " + config.name,
      error: toastErrorMessage(
        value ? "Failed to enable" : "Failed to disable",
      ),
    });
  }
</script>

{#snippet popoverContent()}
  <LinkButton href={resolve("/commands/[id]", { id: config.id })}>
    <SettingsIcon /> View
  </LinkButton>
  <Button onclick={onDelete}><DeleteIcon /> Delete</Button>
{/snippet}

<div class="item">
  <ControlledCheckbox checked={selected} onCheckedChange={onToggleSelected} />

  <div class="item__text">
    <a class="item__name" href={resolve("/commands/[id]", { id: config.id })}>
      {config.name}
    </a>
  </div>

  <div class="actions">
    <EnabledSwitch
      checked={config.enabled}
      onCheckedChange={onChangeDisabled}
    />

    <PopoverButton
      content={popoverContent}
      contentProps={{ align: "start", side: "left" }}
    >
      <SolarMenuDotsBold />
    </PopoverButton>
  </div>
</div>

<style>
  .item {
    background-color: #1a1a1a;
    border: 1px solid #2f2f2f;
    border-radius: 5px;

    display: flex;
    justify-content: space-between;
    gap: 1rem;

    padding: 0.5rem;
    align-items: center;
    overflow: hidden;
    height: 60px;
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

  .item__text {
    display: flex;
    flex: auto;
    align-items: center;
    overflow: hidden;
  }

  .actions {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-shrink: 0;
  }
</style>
