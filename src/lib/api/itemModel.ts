import type {
  Item,
  ItemId,
  SoundId,
  CreateItem,
  UpdateItem,
  UpdateOrdering,
  ItemWithImpactSounds,
} from "$lib/api/types";

import { invoke } from "@tauri-apps/api/core";
import { createQuery, createMutation } from "@tanstack/svelte-query";

import { queryClient } from "./client";

const ITEMS_KEY = ["items"];

function createItemKey(id: ItemId) {
  return ["item", id] as const;
}

// -----------------------------------------------------

export async function updateItemOrder(update: UpdateOrdering[]) {
  await invoke("update_item_orderings", { update });

  queryClient.invalidateQueries({ queryKey: ITEMS_KEY });
}

function invalidateItemsList() {
  queryClient.invalidateQueries({ queryKey: ITEMS_KEY });
}

export async function createItem(create: CreateItem, invalidateList = true) {
  const item = await invoke<ItemWithImpactSounds>("create_item", { create });
  const itemKey = createItemKey(item.id);
  queryClient.setQueryData(itemKey, item);
  if (invalidateList) invalidateItemsList();
  return item;
}

export async function bulkCreateItem(creates: CreateItem[]) {
  await Promise.all(creates.map((create) => createItem(create, false)));

  invalidateItemsList();
}

export function getItemById(itemId: ItemId) {
  return invoke<ItemWithImpactSounds | null>("get_item_by_id", { itemId });
}

export async function updateItem(update: UpdateItem, invalidateList = true) {
  const item = await invoke<ItemWithImpactSounds>("update_item", update);
  const itemKey = createItemKey(item.id);
  queryClient.setQueryData(itemKey, item);

  if (invalidateList) invalidateItemsList();

  return item;
}

export async function updateItems(updates: UpdateItem[]) {
  await Promise.all(updates.map((update) => updateItem(update, false)));

  invalidateItemsList();
}

export async function deleteItem(itemId: ItemId, invalidateList = true) {
  await invoke<void>("delete_item", { itemId });

  const itemKey = createItemKey(itemId);

  // Cancel any queries for the item and clear the current item data
  queryClient.cancelQueries({ queryKey: itemKey });
  queryClient.setQueryData(itemKey, undefined);

  if (invalidateList) invalidateItemsList();
}

export async function bulkDeleteItems(itemIds: ItemId[]) {
  await Promise.all(itemIds.map((itemId) => deleteItem(itemId, false)));
  invalidateItemsList();
}

export async function bulkAppendItemSounds(
  itemIds: ItemId[],
  soundIds: SoundId[],
) {
  await Promise.all(
    itemIds.map((itemId) =>
      invoke<void>("append_item_impact_sounds", {
        itemId,
        sounds: soundIds,
      }),
    ),
  );

  for (const itemId of itemIds) {
    const itemKey = createItemKey(itemId);
    // Cancel any queries for the item and clear the current item data
    queryClient.cancelQueries({ queryKey: itemKey });
    queryClient.setQueryData(itemKey, undefined);
  }

  for (const itemId of itemIds) {
    // Invalidate the specific item query
    const itemKey = createItemKey(itemId);
    queryClient.invalidateQueries({ queryKey: itemKey });
  }

  // Invalid the list of items
  queryClient.invalidateQueries({ queryKey: ITEMS_KEY });
}

// -----------------------------------------------------

export function createItemQuery(idParam: () => ItemId) {
  return createQuery(() => {
    const id = idParam();
    return {
      queryKey: createItemKey(id),
      queryFn: () => getItemById(id),
    };
  });
}

export function createItemsQuery() {
  return createQuery(() => ({
    queryKey: ITEMS_KEY,
    queryFn: () => invoke<Item[]>("get_items"),
  }));
}

export function deleteItemMutation() {
  return createMutation<void, Error, ItemId>(() => ({
    mutationFn: (itemId) => deleteItem(itemId),
  }));
}
