import { BACKEND_HTTP } from "../constants";
import { base64ArrayBuffer } from "../utils/base64";
import { CalibrationStepData } from "./calibration-types";
import { ModelData, OverlayConfig, RuntimeAppData } from "./types";

export async function getOverlayConfig(): Promise<OverlayConfig> {
  const response = await fetch(new URL("/overlay/config", BACKEND_HTTP));
  const json: OverlayConfig = await response.json();
  return json;
}

export async function notifyProgressCalibration(body: CalibrationStepData) {
  const response = await fetch(new URL("/calibration", BACKEND_HTTP), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });

  return response.json();
}

export async function getCalibrationData(): Promise<ModelData[]> {
  const response = await fetch(new URL("/calibration", BACKEND_HTTP));
  return response.json();
}

export async function updateRuntimeData(body: Partial<RuntimeAppData>) {
  try {
    await fetch(new URL("/overlay/data", BACKEND_HTTP), {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
  } catch (e) {
    console.error("failed to set runtime data", e);
  }
}

export async function getVTFTKLogo(): Promise<string> {
  const response = await fetch(new URL("/overlay/icon", BACKEND_HTTP));
  const arrayBuffer = await response.arrayBuffer();
  return base64ArrayBuffer(arrayBuffer);
}

export async function getPersistedAuthToken(): Promise<string | null> {
  const response = await fetch(new URL("/data/vt-auth-token", BACKEND_HTTP));
  const data = await response.json();
  return data.auth_token ?? null;
}

export async function setPersistedAuthToken(token: string | null) {
  await fetch(new URL("/data/vt-auth-token", BACKEND_HTTP), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      auth_token: token,
    }),
  });
}
