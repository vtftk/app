<script lang="ts">
  import SolarRefreshBold from "~icons/solar/refresh-bold";
  import Button from "$lib/components/input/Button.svelte";
  import FormSelect from "$lib/components/form/FormSelect.svelte";
  import { refreshRedeemsList, createGetRedeemsList } from "$lib/api/twitch";

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

  const redeemsList = createGetRedeemsList();

  const items = $derived(
    (redeemsList.data ?? []).map((item) => ({
      value: item.id,
      label: item.title,
      description: item.prompt,
    })),
  );
</script>

<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
{#snippet twitchRedeemItem(item: any)}
  <div class="text-stack item">
    <p class="text-stack--top">{item.label}</p>
    <p class="text-stack--bottom">{item.description}</p>
  </div>
{/snippet}

<div class="container">
  <FormSelect
    {id}
    {name}
    {label}
    {items}
    item={twitchRedeemItem}
    {selected}
    {onChangeSelected}
  />

  <Button type="button" onclick={refreshRedeemsList}>
    <SolarRefreshBold />
    Refresh Redeems
  </Button>
</div>

{#if description}
  <p class="description">{description}</p>
{/if}

{#if redeemsList.isLoading}
  <div class="skeleton" style="width: 90%; height: 1.5rem; padding: 1rem"></div>
{/if}

<style>
  .container {
    display: flex;
    gap: 1rem;
    align-items: flex-end;
    width: 100%;
  }

  .container :global(.form-input) {
    flex: auto;
  }
  .container :global(.form-input [data-select-trigger]) {
    height: 2.65rem;
  }

  .description {
    font-size: 0.8rem;
    color: #999;
    margin-top: 0.5rem;
  }
</style>
