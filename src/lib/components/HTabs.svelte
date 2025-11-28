<script lang="ts">
  import type { Snippet, Component } from "svelte";

  import { Tabs } from "bits-ui";
  import { fly } from "svelte/transition";

  type Props = {
    tabs: Tab[];
  };

  type Tab = {
    value: string;
    icon?: Component;
    label: string;
    content: Snippet;

    // Disable padding around the content
    disablePadding?: boolean;
  };

  const { tabs }: Props = $props();

  const firstTabValue = $derived(tabs.length > 0 ? tabs[0].value : undefined);

  let value: string = $derived(firstTabValue ?? "");
</script>

<Tabs.Root bind:value>
  {#snippet child({ props })}
    <div {...props} class="root">
      <Tabs.List>
        {#snippet child({ props })}
          <div {...props} class="tabs-list">
            {#each tabs as tab (tab.value)}
              <Tabs.Trigger value={tab.value}>
                {#snippet child({ props })}
                  <button
                    {...props}
                    class="tab-button"
                    class:tab-button--active={value === tab.value}
                  >
                    {#if tab.icon}
                      <tab.icon />
                    {/if}
                    {tab.label}
                  </button>
                {/snippet}
              </Tabs.Trigger>
            {/each}
          </div>
        {/snippet}
      </Tabs.List>
      {#each tabs as tab (tab.value)}
        <Tabs.Content value={tab.value}>
          {#snippet child({ props })}
            {#if value === tab.value}
              <div {...props} class="content">
                <div
                  class="content__inner"
                  class:content--disable-padding={tab.disablePadding}
                  in:fly={{ x: -100, duration: 250 }}
                >
                  {@render tab.content()}
                </div>
              </div>
            {/if}
          {/snippet}
        </Tabs.Content>
      {/each}
    </div>
  {/snippet}
</Tabs.Root>

<style>
  .root {
    display: contents;
    height: 100%;
    display: flex;
    flex-flow: column;
  }

  .tabs-list {
    display: flex;
    flex-flow: row;
    justify-content: flex-start;
  }

  .content {
    position: relative;
    flex: auto;
    overflow: auto;
    border: 1px solid #333;
  }

  .content__inner {
    position: relative;
    height: 100%;
    overflow: auto;
    padding: 1rem;
  }

  .content--disable-padding {
    padding: 0;
  }

  .tab-button {
    padding: 0.5rem 0.75rem;
    background-color: #222;
    border: none;
    border-bottom: 1px solid #666;
    color: #fff;
    align-items: center;
    display: flex;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 1em;
    text-decoration: none;
    transition: all 0.25s ease;
    max-width: 100%;
    white-space: nowrap;
  }

  .tab-button:first-of-type {
    border-top-left-radius: 0.25rem;
  }

  .tab-button:last-of-type {
    border-top-right-radius: 0.25rem;
  }

  .tab-button--active {
    background-color: #444;
    border-bottom-color: #888;
  }
</style>
