import { invoke } from "@tauri-apps/api/core";

import type { StorageFolder, VTubeStudioBroadcast } from "./types";

/**
 * Upload a file to the backend file API
 *
 * @param folder Type of file (Determines the file path)
 * @param file The file to upload
 * @returns URL pointing to the uploaded file
 */
export async function uploadFile(
  folder: StorageFolder,
  file: File,
): Promise<string> {
  const name = file.name;
  const data = await file.arrayBuffer();

  return invoke<string>("upload_file", {
    folder,
    name,
    data,
  });
}

export function detectVTubeStudio() {
  return invoke<VTubeStudioBroadcast>("detect_vtube_studio");
}

export function getChatHistoryEstimateSize() {
  return invoke<number>("get_chat_history_estimate_size");
}

export function getExecutionsEstimateSize() {
  return invoke<number>("get_executions_estimate_size");
}

export function getLogsEstimateSize() {
  return invoke<number>("get_logs_estimate_size");
}

export function getServerPort() {
  return invoke<number>("get_http_port");
}
