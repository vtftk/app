<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  import FormErrorLabel from "./FormErrorLabel.svelte";

  type Props = {
    id?: string;
    name: string;
    label: string;
    description?: string;
  } & Omit<HTMLInputAttributes, "name" | "id" | "type">;

  const { name, id = name, label, description, ...props }: Props = $props();
</script>

<div class="form-input">
  <label for={id}>{label}</label>
  <input
    {...props}
    data-felte-keep-on-remove
    type="number"
    {id}
    {name}
    aria-describedby="{name}-validation"
  />
  {#if description}
    <p class="description">{description}</p>
  {/if}
  <FormErrorLabel {name} />
</div>

<style>
  .form-input {
    display: inline-flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .form-input label {
    font-size: 1rem;
    color: #fff;
  }

  .form-input input {
    padding: 0.5rem;
    background-color: #000;
    border: 1px solid #666;
    color: #fff;
    border-radius: 0.25rem;
    align-items: center;
    display: flex;
    gap: 0.5rem;
  }

  .description {
    font-size: 0.8rem;
    color: #999;
  }
</style>
