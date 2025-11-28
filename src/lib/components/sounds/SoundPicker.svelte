<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Sound, SoundId } from "$lib/api/types";

  import getBackendURL from "$lib/utils/url";
  import { filterNameSearch } from "$lib/utils/search";
  import { createSoundsQuery } from "$lib/api/soundModel";
  import { getAppContext } from "$lib/api/runtimeAppData";
  import Button from "$lib/components/input/Button.svelte";
  import Dialog from "$lib/components/dialog/Dialog.svelte";
  import SearchInput from "$lib/components/form/SearchInput.svelte";
  import SoundPreview from "$lib/components/sounds/SoundPreview.svelte";
  import DialogCloseButton from "$lib/components/dialog/DialogCloseButton.svelte";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";

  import BulkSoundImport from "./BulkSoundImport.svelte";
  import { getServerContext } from "../ServerProvider.svelte";

  type Props = {
    disabled?: boolean;
    buttonContent?: Snippet;
    addButtonLabel?: string;

    title?: string;
    description: string;

    selected: SoundId[];
    onChangeSelected: (sounds: Sound[]) => void;
  };

  const {
    disabled,
    buttonContent,
    addButtonLabel = "Done",
    title: titleLabel = "Select Sounds",
    description: descriptionLabel,
    selected: initialSelected,
    onChangeSelected,
  }: Props = $props();

  let search = $state("");
  let selected: SoundId[] = $derived(initialSelected);

  const serverContext = getServerContext();
  const appContext = getAppContext();
  const appData = $derived(appContext.appData);

  const soundsQuery = createSoundsQuery();
  const sounds = $derived(filterNameSearch($soundsQuery.data ?? [], search));

  const onSelectSound = (sound: Sound) => {
    if (selected.includes(sound.id)) {
      selected = selected.filter((id) => id !== sound.id);
    } else {
      selected = [...selected, sound.id];
    }
  };

  const onToggleAll = () => {
    if (sounds.length > 0 && selected.length === sounds.length) {
      selected = [];
    } else {
      selected = sounds.map((sound) => sound.id);
    }
  };

  const onSave = () => {
    onChangeSelected(sounds.filter((sound) => selected.includes(sound.id)));
  };
</script>

<Dialog>
  {#snippet button({ props })}
    <Button {...props} {disabled} type="button">
      {#if buttonContent}
        {@render buttonContent()}
      {:else}
        {initialSelected.length > 0
          ? `${initialSelected.length} Sounds selected`
          : "Select Sounds"}
      {/if}
    </Button>
  {/snippet}

  {#snippet title()}
    {titleLabel}
  {/snippet}

  {#snippet description()}
    {descriptionLabel}
  {/snippet}

  {#snippet actions()}
    <BulkSoundImport label="Import Sounds" />

    <DialogCloseButton buttonLabel={{ text: "Close" }} />
    <DialogCloseButton
      buttonLabel={{ text: addButtonLabel }}
      onclick={onSave}
    />
  {/snippet}

  <div class="selection">
    <SearchInput bind:value={search} placeholder="Search" />
  </div>

  <div class="sound-table-wrapper">
    <table class="sound-table">
      <thead>
        <tr>
          <th class="sound-column sound-column--checkbox">
            <ControlledCheckbox
              id="terms"
              aria-labelledby="terms-label"
              checked={sounds.length > 0 && selected.length === sounds.length}
              onCheckedChange={onToggleAll}
            />
          </th>
          <th class="sound-column sound-column--name">Sound Name</th>
          <th class="sound-column sound-column--preview">Preview</th>
        </tr>
      </thead>
      <tbody>
        {#each sounds as sound (sound.id)}
          <tr class="sound-row">
            <td class="sound-column sound-column--checkbox">
              <ControlledCheckbox
                id="terms"
                aria-labelledby="terms-label"
                checked={selected.includes(sound.id)}
                onCheckedChange={() => onSelectSound(sound)}
              />
            </td>

            <td class="sound-column sound-column--name"> {sound.name} </td>

            <td class="sound-column sound-column--preview">
              <SoundPreview
                volume={sound.volume * appData.sounds_config.global_volume}
                src={getBackendURL(serverContext, sound.src)}
              />
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</Dialog>

<style>
  .selection {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding-left: 1rem;
    padding-right: 1rem;
    padding-bottom: 1rem;
  }

  .sound-table-wrapper {
    max-height: 300px;
    min-height: 300px;
    overflow-y: auto;
    width: 100%;
  }

  .sound-table {
    width: 100%;
    border-collapse: collapse;
  }

  .sound-table tr {
    border: 1px solid #333;
  }

  .sound-table thead {
    position: sticky;
    top: -25px;
    z-index: 1;
    background-color: #111;
  }

  .sound-table td,
  .sound-table th {
    padding: 0.5rem 0.25rem;
  }

  .sound-table .sound-column--checkbox {
    padding-left: 1rem;
    padding-right: 0rem;
  }

  .sound-table .sound-column--preview {
    padding-right: 1rem;
  }

  .sound-table th {
    text-align: left;
    height: 2.5rem;
  }

  .sound-table td:nth-last-child(1),
  .sound-table th:nth-last-child(1) {
    text-align: right;
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }
</style>
