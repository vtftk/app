<script lang="ts">
  import getBackendURL from "$lib/utils/url";

  type Props = {
    id: string;
    name: string;
    scale?: number;
    pixelated?: boolean;

    // Existing source URL
    value?: File | string;

    onChange: (file: File | null) => void;
  };

  const { id, name, value, pixelated, scale, onChange }: Props = $props();

  let inputElm: HTMLInputElement | undefined = $state();
  let currentImage = $state(
    value instanceof File ? URL.createObjectURL(value) : value,
  );

  let dragging = $state(false);

  /**
   * Handle updates to the current image to
   * update the previews
   */
  function onChangeImage() {
    if (!inputElm) return;

    const files = inputElm.files;
    if (!files) return;

    const file = files.item(0);

    onChangeInner(file);
  }

  function onChangeInner(file: File | null) {
    if (file) {
      currentImage = URL.createObjectURL(file);
    } else {
      currentImage = undefined;
    }

    onChange(file);
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    dragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    dragging = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;

    const files = event.dataTransfer?.files;
    const file = files && files.length > 0 ? files[0] : null;
    if (file !== null) {
      onChangeInner(file);
    }
  }
</script>

<button
  class="container"
  class:container--dropping={dragging}
  data-active={currentImage !== undefined}
  type="button"
  onclick={() => {
    inputElm?.click();
  }}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
>
  <div class="image-preview-wrapper">
    {#if currentImage !== undefined}
      <div class="image-preview-container">
        <img
          class="image-preview"
          class:image-preview--pixelate={pixelated}
          src={getBackendURL(currentImage)}
          alt="Preview"
          style="transform: scale({scale});"
        />
      </div>
    {/if}
  </div>

  <input
    data-felte-keep-on-remove
    bind:this={inputElm}
    hidden
    style="display: none;"
    type="file"
    aria-describedby="{name}-validation"
    onchange={onChangeImage}
    accept="image/*"
    {id}
    {name}
  />

  <span class="button">
    {#if value}
      {#if dragging}
        Drop file to replace
      {:else}
        Click to replace
      {/if}
    {:else if dragging}
      Drop file to upload
    {:else}
      Click to upload image
    {/if}
  </span>
</button>

<style>
  .container {
    position: relative;
    height: 300px;
    width: 400px;
    background-color: #000;
    overflow: hidden;
    border: 1px solid #666;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .container--dropping {
    background-color: #333;
    border: 1px solid #999;
  }

  .button {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    color: #fff;
    transition: all 0.25s ease;
  }

  .container[data-active="true"] .button {
    opacity: 0.25;
    top: unset;
    bottom: 1rem;
    transform: translateX(-50%);
  }

  .container[data-active="true"]:hover .button {
    opacity: 1;
  }

  .image-preview-container {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);

    width: auto;
    height: auto;
    max-width: none;
    max-height: none;
  }

  .image-preview {
    display: block;
    width: auto;
    height: auto;
    max-width: none;
    max-height: none;
  }

  .image-preview--pixelate {
    image-rendering: pixelated;
  }
</style>
