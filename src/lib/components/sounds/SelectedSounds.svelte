<script lang="ts">
  import { createSoundsQuery } from "$lib/api/soundModel";

  type Props = {
    soundIds: string[];
  };

  const { soundIds }: Props = $props();

  const soundsQuery = createSoundsQuery();
  const sounds = $derived($soundsQuery.data ?? []);

  const selectedOptions = $derived(
    sounds.filter((sound) => soundIds.includes(sound.id)),
  );
</script>

<div class="grid">
  {#each selectedOptions as option (option.id)}
    <li class="item">
      <p class="item__name">{option.name}</p>
    </li>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    overflow: hidden;
    gap: 1rem;
  }

  .item {
    width: 100%;
    overflow: hidden;
    background-color: #222;
    color: #ccc;
    text-align: left;

    display: block;
    padding: 0.5rem;
    border: 1px solid #333;

    border-radius: 0.25rem;
  }

  .item__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
