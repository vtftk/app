<script
  lang="ts"
  generics="V extends string, T extends { value: V, label: string }"
>
  import type { Snippet } from "svelte";

  import { Select, type SelectRootProps } from "bits-ui";
  import SolarAltArrowUpBold from "~icons/solar/alt-arrow-up-bold";
  import SolarAltArrowDownBold from "~icons/solar/alt-arrow-down-bold";

  import Button from "../input/Button.svelte";
  import FormErrorLabel from "./FormErrorLabel.svelte";

  type Props<T> = {
    id: string;
    name: string;
    label: string;
    description?: string;

    items: T[];
    item: Snippet<[T]>;

    selected: V | undefined;

    onChangeSelected: (value: V) => void;
  } & Omit<
    SelectRootProps,
    "type" | "onOpenChange" | "value" | "onValueChange"
  >;

  const {
    id,
    name,
    label,
    description,
    items,
    item,
    selected: selectedValues,
    onChangeSelected,
    allowDeselect,
    ...rootProps
  }: Props<T> = $props();

  let open = $state(false);

  function getSelection(
    items: T[],
    selected: string | undefined,
  ): { value: string; label: string } | undefined {
    const item = items.find((item) => item.value === selected);
    if (item === undefined) return undefined;
    return {
      value: item.value,
      label: item.label,
    };
  }

  const selection = $derived(getSelection(items, selectedValues));

  const selectedLabel = $derived.by(() => {
    if (selection === undefined) return undefined;
    return selection.label;
  });
</script>

<!-- eslint-disable @typescript-eslint/no-explicit-any -->

<div class="form-input">
  <label for={id}>{label}</label>
  <Select.Root
    {...rootProps}
    allowDeselect={allowDeselect ?? false}
    type="single"
    onOpenChange={(value) => {
      open = value;
    }}
    value={selectedValues}
    onValueChange={(selection: any) => onChangeSelected(selection)}
  >
    <Select.Trigger>
      {#snippet child({ props })}
        <Button {...props}>
          {#if open}
            <SolarAltArrowUpBold />
          {:else}
            <SolarAltArrowDownBold />
          {/if}

          {selectedLabel}
        </Button>
      {/snippet}
    </Select.Trigger>

    <Select.Portal>
      <Select.Content sideOffset={8}>
        {#snippet child({ props, open, wrapperProps })}
          <div {...wrapperProps} class="content-wrapper">
            {#if open}
              <div {...props} class="content">
                {#each items as value, index (index)}
                  <Select.Item value={value.value} label={value.label}>
                    {#snippet child({ props, selected, highlighted })}
                      <div
                        {...props}
                        class="item"
                        class:item--selected={selected}
                        class:item--highlighted={highlighted}
                      >
                        {@render item(value)}
                      </div>
                    {/snippet}
                  </Select.Item>
                {/each}
              </div>
            {/if}
          </div>
        {/snippet}
      </Select.Content>
    </Select.Portal>
  </Select.Root>

  {#if description}
    <p class="description">{description}</p>
  {/if}

  <FormErrorLabel {name} />
</div>

<style>
  .content {
    border-radius: 0.5rem;
    border: 0.125rem solid #888;
    background-color: #444;
    padding: 0.25rem;
    max-height: 40vh;
    overflow: auto;
    box-shadow: 2px 10px 20px rgba(0, 0, 0, 0.1);
  }

  .form-input {
    display: inline-flex;
    flex-flow: column;
  }

  .form-input label {
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  .description {
    font-size: 0.8rem;
    color: #999;
    margin-top: 0.5rem;
  }

  .item {
    cursor: pointer;
    display: flex;
    border-radius: 0.25rem;
    padding: 0.5rem;
    max-width: 500px;
  }

  .item:hover {
    background-color: #777;
  }

  .item--selected {
    background-color: #888;
  }

  .item--highlighted {
    outline: 1px solid white;
  }
</style>
