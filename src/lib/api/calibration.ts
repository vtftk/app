import type { ModelData } from "$lib/api/types";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createQuery } from "@tanstack/svelte-query";

import { queryClient } from "./client";

export enum CalibrationStep {
  NotStarted = "NotStarted",

  // Original model position is known, we have shrunk to the smallest
  // size preparing to capture the min face X Y
  Smallest = "Smallest",

  // Smallest model position is known, we have grown to the largest
  // size preparing to capture the max face X Y
  Largest = "Largest",

  // Both positions are known
  Complete = "Complete",
}

const CALIBRATION_DATA_KEY = ["calibration-data"];

export function createModelDataQuery() {
  return createQuery(() => ({
    queryKey: CALIBRATION_DATA_KEY,
    queryFn: () => invoke<ModelData[]>("get_calibration_data"),
  }));
}

// Handle calibration data change
listen<ModelData>("model_data_updated", ({ payload: modelData }) => {
  queryClient.setQueryData<ModelData[]>(
    CALIBRATION_DATA_KEY,
    (existingModelData) => {
      if (existingModelData === undefined) return undefined;

      return [
        ...existingModelData.filter((data) => data.id !== modelData.id),
        modelData,
      ];
    },
  );
});
