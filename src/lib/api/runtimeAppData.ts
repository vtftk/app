import { getContext } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  createQuery,
  createMutation,
  type CreateQueryResult,
} from "@tanstack/svelte-query";

import type { AppData, ModelData, RuntimeAppData } from "./types";

import { queryClient } from "./client";

export const APP_CONTEXT_KEY = Symbol("app-context");

export const RUNTIME_APP_DATA_KEY = ["runtime-app-data"];
export const APP_DATA_KEY = ["app-data"];

// Update the runtime app data when the change event is received
listen<RuntimeAppData>("runtime_app_data_changed", ({ payload }) => {
  queryClient.cancelQueries({ queryKey: RUNTIME_APP_DATA_KEY });
  queryClient.setQueryData(RUNTIME_APP_DATA_KEY, payload);
});

type AppContext = {
  appData: AppData;
  runtimeAppData: RuntimeAppData;
};

export function getAppContext(): AppContext {
  return getContext(APP_CONTEXT_KEY);
}

/**
 * Create a query to fetch the runtime app data
 */
export function createRuntimeAppDataQuery(): CreateQueryResult<
  RuntimeAppData,
  Error
> {
  return createQuery(() => ({
    queryKey: RUNTIME_APP_DATA_KEY,
    queryFn: () => invoke<RuntimeAppData>("get_runtime_app_data"),
  }));
}

/**
 * Create a query to fetch the app data
 */
export function createAppDataQuery(): CreateQueryResult<AppData, Error> {
  return createQuery(() => ({
    queryKey: APP_DATA_KEY,
    queryFn: () => invoke<AppData>("get_app_data"),
  }));
}

export function createAppDateMutation() {
  return createMutation<boolean, Error, AppData>(() => ({
    mutationFn: (appData) => invoke<boolean>("set_app_data", { appData }),
    onMutate: async (appData) => {
      queryClient.cancelQueries({ queryKey: APP_DATA_KEY });
      queryClient.setQueryData(APP_DATA_KEY, appData);
      return appData;
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: APP_DATA_KEY });
    },
  }));
}

/**
 * Create a query to load the overlay URL
 */
export function createOverlayURLQuery() {
  return createQuery(() => ({
    queryKey: ["overlay-url"],
    queryFn: () => invoke<string>("get_overlay_url"),
  }));
}

export function getTwitchOAuthURI() {
  return invoke<string>("get_twitch_oauth_uri");
}

/**
 * Checks if the current model is present in the
 * provided model calibration data
 *
 * @param modelData Available model data
 * @param modelId ID of the current model or null when no model
 * @returns Whether the model is calibrated
 */
export function isModelCalibrated(
  modelData: ModelData[],
  modelId: string | null,
): boolean {
  // No model active or no model data
  if (modelId === null || modelData.length < 1) {
    return false;
  }

  const data = modelData.find((data) => data.id === modelId);
  return data !== undefined;
}
