<script lang="ts">
  import { MinimumRequiredRole } from "$lib/api/types";
  import FormSelect from "$lib/components/form/FormSelect.svelte";

  type Props = {
    id?: string;
    name: string;
    label: string;
    description?: string;

    selected: MinimumRequiredRole;
    onChangeSelected: (value: MinimumRequiredRole) => void;
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
      value: MinimumRequiredRole.None,
      label: "None",
      description: "Anyone can use",
    },
    {
      value: MinimumRequiredRole.Follower,
      label: "Follower",
      description: "Must be following the streamer",
    },
    {
      value: MinimumRequiredRole.Vip,
      label: "VIP",
      description: "Must be VIP, Moderator, or the streamer to use",
    },
    {
      value: MinimumRequiredRole.Mod,
      label: "Moderator",
      description: "Must be moderator or the streamer to use",
    },
    {
      value: MinimumRequiredRole.Broadcaster,
      label: "Broadcaster",
      description: "Must be the streamer to use",
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
