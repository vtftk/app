<script lang="ts">
  import type { SoundId } from "$shared/dataV2";

  import { createSoundsQuery } from "$lib/api/soundModel";
  import FormSelect from "$lib/components/form/FormSelect.svelte";

  type Props = {
    id: string;
    name: string;
    label: string;
    description?: string;

    selected: SoundId;
    onChangeSelected: (value: SoundId) => void;
  };

  const { id, name, label, description, selected, onChangeSelected }: Props =
    $props();

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
  Loading sounds...
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
