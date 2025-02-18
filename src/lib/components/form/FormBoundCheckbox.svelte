<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  import FormErrorLabel from "./FormErrorLabel.svelte";

  type Props = {
    id?: string;
    name: string;

    label: string;
    description?: string;
  } & Omit<HTMLInputAttributes, "name" | "id" | "type" | "hidden">;

  const { name, id = name, label, description, ...props }: Props = $props();
</script>

<div class="form-input">
  <!-- Hidden element -->
  <input
    {...props}
    data-felte-keep-on-remove
    hidden
    type="checkbox"
    {id}
    {name}
    aria-labelledby="{name}-label"
  />

  <!-- Checkbox -->
  <label for={id} class="checkbox">
    <span class="checkbox__indicator">&#10003; </span>
  </label>

  <!-- Label -->
  <label for={id}>
    <span class="label">{label}</span>
    {#if description}
      <p class="description">{description}</p>
    {/if}
  </label>

  <!-- Validation -->
  <FormErrorLabel {name} />
</div>

<style>
  .form-input {
    display: inline-flex;
    flex-flow: row;
    gap: 0.75rem;
    align-items: center;
  }

  .label {
    font-size: 1rem;
    color: #fff;
    display: block;
  }

  .description {
    font-size: 0.9rem;
    color: #ccc;
  }

  .checkbox {
    display: inline-flex;
    width: 32px;
    height: 32px;
    align-items: center;
    justify-content: center;
    border-radius: 0.375rem;
    border: 1px solid #444;
    background-color: #333;
    transition: all 150ms ease-in-out;
    cursor: pointer;
  }

  .checkbox__indicator {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    width: 100%;
    height: 100%;
    opacity: 0;
  }

  /* Checkbox is checked */
  .form-input input:checked + .checkbox > .checkbox__indicator {
    opacity: 1;
  }

  /* Checkbox is checked */
  .form-input input:checked + .checkbox {
    background-color: #666;
    border: 1px solid #777;
  }

  /* Checkbox not checked is hovered */
  .form-input input:not(:checked) + .checkbox:hover {
    border: 1px solid #fff;
  }

  /* Checkbox is checked and hovered */
  .form-input input:checked + .checkbox:hover {
    border: 1px solid #fff;
  }
</style>
