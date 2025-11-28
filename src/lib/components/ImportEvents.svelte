<script lang="ts">
  import ImportIcon from "~icons/solar/import-bold";
  import { importEvents } from "$lib/api/eventModel";
  import Button from "$lib/components/input/Button.svelte";

  let inputElm: HTMLInputElement | undefined = $state();

  async function onChangeFile() {
    if (!inputElm) return;

    const files = inputElm.files;
    if (!files) return;

    const file = files.item(0);
    if (!file) return;

    if (inputElm) clearFileInput(inputElm);

    const content = await readFile(file);
    const parsed = JSON.parse(content);
    await importEvents(parsed);
  }

  function clearFileInput(ctrl: HTMLInputElement) {
    try {
      ctrl.value = "";
    } catch {
      //
    }

    if (ctrl.value) {
      ctrl.parentNode?.replaceChild(ctrl.cloneNode(true), ctrl);
    }
  }

  function readFile(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = () => {
        try {
          resolve(reader.result as string);
        } catch (error) {
          reject(error);
        }
      };

      reader.onerror = () => {
        reject(new Error("Failed to read file"));
      };

      reader.readAsText(file);
    });
  }
</script>

<Button
  onclick={() => {
    inputElm?.click();
  }}
>
  <ImportIcon /> Import
</Button>

<input
  bind:this={inputElm}
  hidden
  multiple
  style="display: none;"
  type="file"
  onchange={onChangeFile}
  accept="application/json"
/>

<style>
</style>
