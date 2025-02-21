<script lang="ts" generics="T extends { id: string }">
  import type { Snippet } from "svelte";
  import type { UpdateOrdering } from "$lib/api/types";

  import { onMount } from "svelte";
  import { passiveEventArg } from "$lib/utils/browser";
  import {
    dndzone,
    type DndEvent,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    type TransformDraggedElementFunction,
  } from "svelte-dnd-action";

  type Props = {
    // Available items for the grid
    items: T[];

    // Snippet for rendering items
    item: Snippet<[T]>;

    itemHeight?: number;

    // Called when the ordering of the list is due for an update
    onUpdateOrder: (ordering: UpdateOrdering[]) => Promise<void>;

    // Optionally disable ordering when set
    disableOrdering?: boolean;
  };

  const {
    items: _items,
    item: renderItem,
    itemHeight = 68,
    onUpdateOrder,
    disableOrdering,
  }: Props = $props();

  let containerHeight: number = $state(0);
  let wrapper: HTMLDivElement | undefined = $state();
  let wrapperStyle = $state("");

  // Local state for list of items to allow reordering
  let items: T[] = $state([]);

  // Update the items when the props change
  $effect(() => {
    items = _items;
  });

  const styleCache: Partial<Record<number, string>> = {};
  const rowCount = $derived(Math.ceil(items.length / 2));
  const totalSize = $derived(rowCount * itemHeight);

  // Current scroll offset
  let offset = $state(0);

  const virtualItems: VirtualItems = $derived.by(() =>
    createVirtualItems(containerHeight, offset, rowCount, itemHeight),
  );

  type VirtualItems = {
    startIndex: number;
    stopIndex: number;
  };

  const COLUMNS: number = 2;

  onMount(() => {
    if (wrapper === undefined) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        containerHeight = entry.target.clientHeight;
      }
    });

    containerHeight = wrapper.clientHeight;

    resizeObserver.observe(wrapper);
    wrapper.addEventListener("scroll", handleScroll, passiveEventArg);

    return () => {
      if (wrapper === undefined) return;
      wrapper.removeEventListener("scroll", handleScroll);
    };
  });

  function binarySearch({
    low,
    high,
    offset,
  }: {
    low: number;
    high: number;
    offset: number;
  }) {
    let middle = 0;
    let currentOffset = 0;

    while (low <= high) {
      middle = low + Math.floor((high - low) / 2);
      currentOffset = middle * itemHeight;

      if (currentOffset === offset) {
        return middle;
      } else if (currentOffset < offset) {
        low = middle + 1;
      } else if (currentOffset > offset) {
        high = middle - 1;
      }
    }

    if (low > 0) {
      return low - 1;
    }

    return 0;
  }

  function createVirtualItems(
    containerSize: number,
    offset: number,
    rowCount: number,
    itemHeight: number,
  ): VirtualItems {
    const overscanCount = 2;
    const maxOffset = offset + containerSize;

    const totalSize = rowCount * itemHeight;

    if (totalSize === 0) {
      return {
        startIndex: 0,
        stopIndex: 0,
      };
    }

    // Find the first visible row
    const startRow = Math.max(
      0,
      binarySearch({
        high: rowCount,
        low: 0,
        offset: Math.max(0, offset),
      }) - overscanCount,
    );

    const startOffset = startRow * itemHeight;
    const visibleHeight = maxOffset - startOffset;
    const visibleRows = Math.ceil(visibleHeight / itemHeight);

    // Determine the last visible row
    const stopRow = Math.min(
      startRow + visibleRows + overscanCount,
      rowCount - 1,
    );

    const startIndex: number = startRow;
    const stopIndex: number = stopRow * COLUMNS + COLUMNS - 1;

    return {
      startIndex,
      stopIndex,
    };
  }

  function getItemStyle(index: number) {
    if (styleCache[index]) return styleCache[index];

    // Compute row
    const row = Math.floor(index / 2);

    // Compute column
    const column = index % 2;

    const offset = row * itemHeight;

    let style = `height:${itemHeight}px;position:absolute;top:${offset}px;`;
    if (column === 1) {
      style += "width:calc(50% - 4px);left:calc(50% + 4px);";
    } else {
      style += "left:0;width:calc(50% - 4px)";
    }

    return (styleCache[index] = style);
  }

  function handleScroll(event: Event) {
    const scrollOffset = wrapper?.scrollTop ?? 0;

    if (scrollOffset < 0 || offset === scrollOffset || event.target !== wrapper)
      return;

    offset = scrollOffset;
  }

  function handleDndConsider(e: CustomEvent<DndEvent<T>>) {
    items = e.detail.items;
  }

  async function handleDndFinalize(e: CustomEvent<DndEvent<T>>) {
    // Splice the new collection items
    items = e.detail.items;
    onUpdateOrder(items.map((item, index) => ({ id: item.id, order: index })));
  }

  const transformDraggedElement: TransformDraggedElementFunction = (
    element?: HTMLElement,
    _data?: unknown,
    _index?: number,
  ) => {
    if (element) {
      // @ts-expect-error Clearing left position styling
      element.style.left = undefined;
    }
  };
</script>

<div bind:this={wrapper} class="wrapper" style={wrapperStyle}>
  <div
    style={`height:${totalSize}px;`}
    class="grid"
    use:dndzone={{
      items,
      dragDisabled: disableOrdering,
      transformDraggedElement,
    }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
  >
    {#each items as item, index (item.id)}
      <div style={getItemStyle(index)}>
        {#if index >= virtualItems.startIndex && index <= virtualItems.stopIndex}
          <div class="item-wrapper">
            {@render renderItem(item)}

            <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
            {#if (item as any)[SHADOW_ITEM_MARKER_PROPERTY_NAME]}
              <div class="custom-shadow-item"></div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .item-wrapper {
    position: relative;
  }

  .custom-shadow-item {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    visibility: visible;
    border: 3px dashed #999;
    background: #666;
    opacity: 0.5;
    margin: 0;
    border-radius: 5px;
  }

  .wrapper {
    overflow: auto;
    will-change: transform;
    -webkit-overflow-scrolling: touch;
    height: 100%;
    width: 100%;
    position: relative;
  }
</style>
