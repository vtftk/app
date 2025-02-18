<script lang="ts">
  import type { SoundId } from "$lib/api/types";

  import { createSoundsQuery } from "$lib/api/soundModel";
  import FormSelect from "$lib/components/form/FormSelect.svelte";

  type Props = {
    id?: string;
    name: string;
    label: string;
    description?: string;

    selected: SoundId;
    onChangeSelected: (value: SoundId) => void;
  };

  const {
    name,
    id = name,
    label,
    description,
    selected,
    onChangeSelected,
  }: Props = $props();

  const soundsQuery = createSoundsQuery();

  const items = $derived(
    ($soundsQuery.data ?? []).map((sound) => ({
      value: sound.id,
      label: sound.name,
    })),
  );

  type Option = (typeof items)[0];
</script>

{#if $soundsQuery.isLoading}
  <div class="skeleton" style="width: 90%; height: 1.5rem; padding: 1rem"></div>
{/if}

{#snippet item(item: Option)}
  <div class="text-stack">
    <p class="text-stack--top">{item.label}</p>
  </div>
{/snippet}

<FormSelect
  {id}
  {name}
  {label}
  {description}
  {items}
  {item}
  {selected}
  {onChangeSelected}
/>
