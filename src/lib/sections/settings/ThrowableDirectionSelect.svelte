<script lang="ts">
  import { ThrowDirection } from "$lib/api/types";
  import FormSelect from "$lib/components/form/FormSelect.svelte";

  type Props = {
    id: string;
    name: string;
    label: string;
    description?: string;

    selected: ThrowDirection;
    onChangeSelected: (value: ThrowDirection) => void;
  };

  const { id, name, label, description, selected, onChangeSelected }: Props =
    $props();

  const options = [
    {
      value: ThrowDirection.Weighted,
      label: "Weighted",
      description:
        "Randomly pick between the left and right side with a higher chance to come from the opposite side to the model",
    },
    {
      value: ThrowDirection.Random,
      label: "Random",
      description: "Randomly pick between the left and right side",
    },
    {
      value: ThrowDirection.LeftOnly,
      label: "Left Only",
      description: "Only throw from the left side",
    },
    {
      value: ThrowDirection.RightOnly,
      label: "Right Only",
      description: "Only throw from the right side",
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
