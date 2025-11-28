import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createQuery, type CreateQueryResult } from "@tanstack/svelte-query";

import type { CustomReward } from "./types";

import { queryClient } from "./client";

export const GET_REDEEMS_LIST_KEY = ["get-redeems-list"];
export const IS_AUTHENTICATED_KEY = ["is-authenticated"];

// -----------------------------------------------------

/**
 * Create a query to fetch the runtime app data
 */
export function createGetRedeemsList(): CreateQueryResult<
  CustomReward[],
  Error
> {
  return createQuery(() => ({
    queryKey: GET_REDEEMS_LIST_KEY,
    queryFn: () => invoke<CustomReward[]>("get_redeems_list"),
  }));
}

export async function refreshRedeemsList() {
  await invoke("refresh_redeems_list");

  queryClient.cancelQueries({ queryKey: GET_REDEEMS_LIST_KEY });
  queryClient.invalidateQueries({ queryKey: GET_REDEEMS_LIST_KEY });
}

/**
 * Create a query to fetch the runtime app data
 */
export function createIsAuthenticatedQuery(): CreateQueryResult<
  boolean,
  Error
> {
  return createQuery(() => ({
    queryKey: IS_AUTHENTICATED_KEY,
    queryFn: () => invoke<boolean>("is_authenticated"),
  }));
}

export function logout() {
  return invoke<void>("logout");
}

// Handle authenticating
listen("authenticated", () => {
  queryClient.cancelQueries({ queryKey: IS_AUTHENTICATED_KEY });
  queryClient.setQueryData(IS_AUTHENTICATED_KEY, true);
});

// Handle logout
listen("logout", () => {
  queryClient.cancelQueries({ queryKey: IS_AUTHENTICATED_KEY });
  queryClient.setQueryData(IS_AUTHENTICATED_KEY, false);
});
