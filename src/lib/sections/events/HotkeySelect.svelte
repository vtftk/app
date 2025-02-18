<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getAppContext } from "$lib/api/runtimeAppData";
  import SolarRefreshBold from "~icons/solar/refresh-bold";
  import Button from "$lib/components/input/Button.svelte";
  import FormSelect from "$lib/components/form/FormSelect.svelte";

  type Props = {
    id?: string;
    name: string;
    label: string;
    description?: string;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    selected: any;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onChangeSelected: (value: any) => void;
  };

  const {
    label,
    name,
    id = name,
    description,
    selected,
    onChangeSelected,
  }: Props = $props();

  const updateHotkeys = () => {
    invoke("update_hotkeys");
  };

  const appContext = getAppContext();
  const runtimeAppData = $derived(appContext.runtimeAppData);

  onMount(() => {
    updateHotkeys();
  });

  const items = $derived.by(() => {
    const options = runtimeAppData.hotkeys.map((item) => ({
      value: item.hotkey_id,
      label: item.name,
    }));

    if (options.length === 0 && selected) {
      options.push({
        value: selected,
        label: `Unknown Hotkey (${selected})`,
      });
    }

    return options;
  });
</script>

{#snippet item(item: (typeof items)[0])}
  <div class="text-stack">
    <p class="text-stack--top">{item.label}</p>
  </div>
{/snippet}

<div>
  <div class="container">
    <FormSelect
      {id}
      {name}
      {label}
      {items}
      {item}
      {selected}
      {onChangeSelected}
    />

    <Button type="button" onclick={updateHotkeys}>
      <SolarRefreshBold /> Refresh Hotkeys
    </Button>
  </div>

  {#if !runtimeAppData.vtube_studio_connected}
    <p class="error">
      Not connected to VTube studio... Connect to see available Hotkeys
    </p>
  {/if}

  {#if description}
    <p class="description">{description}</p>
  {/if}
</div>

<style>
  .container {
    display: flex;
    gap: 0.5rem;
    align-items: flex-end;
    width: 100%;
  }

  .error {
    color: #dba33a;
    font-size: 0.8rem;
    margin-top: 0.5rem;
  }

  .description {
    font-size: 0.8rem;
    color: #999;
    margin-top: 0.5rem;
  }

  .container :global(.form-input) {
    flex: auto;
  }
  .container :global(.form-input [data-select-trigger]) {
    height: 2.65rem;
  }
</style>
