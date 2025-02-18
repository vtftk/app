<script lang="ts">
  import { ThrowableDataType } from "$lib/api/types";
  import FormSelect from "$lib/components/form/FormSelect.svelte";

  type Props = {
    id?: string;
    name: string;
    label: string;
    description?: string;

    selected: ThrowableDataType;
    onChangeSelected: (value: ThrowableDataType) => void;
  };

  const {
    name,
    id = name,
    label,
    description,
    selected,
    onChangeSelected,
  }: Props = $props();

  const options = [
    {
      value: ThrowableDataType.Throw,
      label: "Throw",
      description: "Throw selected number of items all at once",
    },
    {
      value: ThrowableDataType.Barrage,
      label: "Barrage",
      description: "Throw items as a barrage of multiple thrown items",
    },
  ];

  type Option = (typeof options)[0];
</script>

{#snippet item(item: Option)}
  <div class="text-stack">
    <p class="text-stack--top">{item.label}</p>
    <p class="text-stack--bottom">{item.description}</p>
  </div>
{/snippet}

<FormSelect
  {id}
  {name}
  {label}
  {description}
  items={options}
  {item}
  {selected}
  {onChangeSelected}
/>
