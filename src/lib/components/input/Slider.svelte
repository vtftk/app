<script lang="ts">
  import { Slider, type WithoutChildren } from "bits-ui";

  type Props = { showTicks?: boolean } & WithoutChildren<
    Omit<Slider.RootProps & { type: "single" }, "type">
  >;

  let {
    value = $bindable(),
    ref = $bindable(null),
    showTicks,
    ...restProps
  }: Props = $props();
</script>

<div class="wrapper">
  <Slider.Root type="single" bind:value bind:ref {...restProps}>
    {#snippet child({ props, thumbs, ticks })}
      <span {...props} class="root">
        <Slider.Range>
          {#snippet child({ props })}
            <span {...props} class="range"></span>
          {/snippet}
        </Slider.Range>
        {#each thumbs as index (index)}
          <Slider.Thumb {index}>
            {#snippet child({ props })}
              <span {...props} class="thumb"></span>
            {/snippet}
          </Slider.Thumb>
        {/each}

        {#if showTicks}
          {#each ticks as index (index)}
            <Slider.Tick {index}>
              {#snippet child({ props })}
                <span {...props} class="tick"></span>
              {/snippet}
            </Slider.Tick>
          {/each}
        {/if}
      </span>
    {/snippet}
  </Slider.Root>
</div>

<style>
  .wrapper {
    padding: 0.75rem 1rem;
    background-color: #333;
    border-radius: 0.25rem;
    border: 1px solid #666;
  }

  .root {
    position: relative;
    display: flex;
    width: 100%;
    touch-action: none;
    user-select: none;
    align-items: center;

    height: 1rem;
  }

  .range {
    position: relative;

    height: 1rem;
    width: 100%;
    overflow: hidden;
    border-radius: 100%;
    flex-grow: 1;
  }

  .thumb {
    background-color: #fff;
    width: 1.5rem;
    height: 1.5rem;
    display: block;
    cursor: pointer;
    border-radius: 100%;
    border: 1px solid #777;
  }

  .tick {
    width: 1px;
    height: 5px;
    background-color: #fff;
    cursor: pointer;
  }
</style>
