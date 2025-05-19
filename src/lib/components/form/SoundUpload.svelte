<script lang="ts">
  import getBackendURL from "$lib/utils/url";

  import Button from "../input/Button.svelte";
  import FormErrorLabel from "./FormErrorLabel.svelte";
  import SoundPreview from "../sounds/SoundPreview.svelte";
  import { getServerContext } from "../ServerProvider.svelte";

  type Props = {
    id: string;
    name: string;
    label: string;

    // Existing source URL
    existing?: string;

    volume?: number;

    onChangeSound: (sound: File | null) => void;
  };

  const { id, name, label, existing, volume, onChangeSound }: Props = $props();

  const serverContext = getServerContext();

  let inputElm: HTMLInputElement | undefined = $state();
  let currentSound = $state(existing);

  /**
   * Handle updates to the current sound to
   * update the previews
   */
  function _onChangeSound() {
    if (!inputElm) return;

    const files = inputElm.files;
    if (!files) return;

    const file = files.item(0);

    if (file) {
      currentSound = URL.createObjectURL(file);
    } else {
      currentSound = undefined;
    }

    onChangeSound(file);
  }
</script>

<div class="form-input">
  <label for={id}>{label}</label>

  <div class="sound-preview-wrapper">
    {#if currentSound !== undefined}
      <SoundPreview src={getBackendURL(serverContext, currentSound)} {volume} />
    {/if}
  </div>

  <Button
    type="button"
    onclick={() => {
      inputElm?.click();
    }}
    >{currentSound ? "Replace" : "Select"} Sound
  </Button>

  <input
    data-felte-keep-on-remove
    bind:this={inputElm}
    hidden
    style="display: none;"
    type="file"
    aria-describedby="{name}-validation"
    onchange={_onChangeSound}
    {id}
    {name}
    accept="audio/*"
  />

  <FormErrorLabel {name} />
</div>

<style>
  .sound-preview-wrapper {
    position: relative;
    margin: 1rem 0;
  }

  .form-input {
    display: inline-block;
    border-radius: 0.5rem;
  }

  .form-input label {
    font-size: 1rem;
  }

  .form-input :global(.btn) {
    width: 100%;
  }
</style>
