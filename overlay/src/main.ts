import "./styles/app.css";
import "./vtftk/events";
import "./vtftk/calibration";
import { RuntimeAppData } from "./vtftk/types";
import { subscribeEvent } from "./vtube-studio/event";
import { attemptAuthorization } from "./vtube-studio/auth";
import { VTubeStudioWebSocket } from "./vtube-studio/socket";
import { EventSourceData, createEventSource } from "./vtftk/events";
import {
  getOverlayConfig,
  updateRuntimeData,
  getCalibrationData,
} from "./vtftk/api";
import {
  requestCurrentModel,
  createModelParameters,
  requestInputParameterList,
} from "./vtube-studio/model";

async function load() {
  // Tell the backend we aren't connected
  await updateRuntimeData({
    model_id: null,
    vtube_studio_connected: false,
    vtube_studio_auth: false,
  });

  const overlayConfig = await getOverlayConfig();

  const eventSourceData: EventSourceData = {
    overlayConfig: overlayConfig,
    modelCalibration: new Map(),
    vtSocket: undefined,
    modelParameters: undefined,
  };

  // Load and store model calibration data
  const modelData = await getCalibrationData();
  modelData.forEach((modelData) =>
    eventSourceData.modelCalibration.set(modelData.id, modelData.calibration),
  );

  const eventSource = createEventSource(eventSourceData);

  const vtSocket = new VTubeStudioWebSocket(
    overlayConfig.vtube_studio_config.host,
    overlayConfig.vtube_studio_config.port,
  );

  eventSourceData.vtSocket = vtSocket;

  // Handle reporting the current app state when the event source is established
  eventSource.addEventListener("open", () => {
    reportCurrentRuntimeData(vtSocket);
  });

  // Run when the socket is connected
  vtSocket.onConnected = async () => {
    vtSocket.setAuthenticated(false);

    // Tell the backend we are connected
    updateRuntimeData({
      model_id: null,
      vtube_studio_connected: true,
      vtube_studio_auth: false,
    });

    // Make a login attempt
    await attemptAuthorization(vtSocket);

    vtSocket.setAuthenticated(true);

    // Tell the backend we are authenticated
    updateRuntimeData({
      vtube_studio_auth: true,
    });

    console.debug(
      "VTube studio authorization complete, Connected to VTube studio",
    );

    onModelReady();

    await subscribeEvent(vtSocket, "ModelLoadedEvent", true, {});
  };

  vtSocket.onModelLoaded = onModelReady;

  async function onModelReady() {
    console.log("Model ready, loading model data");

    const { modelID } = await requestCurrentModel(vtSocket);

    // Update the current active model
    updateRuntimeData({
      model_id: modelID,
    });

    // Only needs to be done on initial load, can be stored until next refresh
    const inputParameters = await requestInputParameterList(vtSocket);
    const modelParameters = createModelParameters(
      inputParameters.defaultParameters,
    );

    eventSourceData.modelParameters = modelParameters;
  }

  vtSocket.onDisconnect = () => {
    // Tell the backend we aren't connected
    updateRuntimeData({
      model_id: null,
      vtube_studio_connected: false,
      vtube_studio_auth: false,
      hotkeys: [],
    });

    vtSocket.setAuthenticated(false);
  };

  vtSocket.connect();
}

async function reportCurrentRuntimeData(vtSocket: VTubeStudioWebSocket) {
  const runtimeData: Partial<RuntimeAppData> = {};

  if (vtSocket.isConnected()) {
    runtimeData.vtube_studio_connected = true;
    runtimeData.vtube_studio_auth = vtSocket.getAuthenticated();

    try {
      const { modelID } = await requestCurrentModel(vtSocket);
      runtimeData.model_id = modelID;
    } catch (e) {
      console.error("failed to request current model", e);
    }
  }

  // Report current state to backend
  updateRuntimeData(runtimeData);
}

load();
