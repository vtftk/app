import { EyesMode } from "../vtftk/types";
import { VTubeStudioWebSocket } from "./socket";
import {
  ModelParameters,
  injectParameterData,
  InjectParameterValue,
} from "./model";

type FlinchReturn = {
  cancel: VoidFunction;
};

const FLINCH_RETURN_ANIMATE_INTERVAL = 1000 / 60;

let currentFlinchReturn: FlinchReturn | undefined;

type FlinchConfig = {
  leftSide: boolean;
  angle: number;
  magnitude: number;
  returnSpeed: number;
  eyeState: EyesMode;
};

export function flinch(
  socket: VTubeStudioWebSocket,
  modelParameters: ModelParameters,
  config: FlinchConfig,
) {
  const parameterValues: InjectParameterValue[] = [];

  for (const param of modelParameters.horizontal) {
    const value = config.leftSide ? param.max : param.min;

    parameterValues.push({
      id: param.name,
      value: value * config.magnitude,
    });
  }

  for (const param of modelParameters.vertical) {
    let value = config.angle > 0 ? param.min : param.max;
    value = (value * Math.abs(config.angle)) / 45;

    parameterValues.push({
      id: param.name,
      value: value * config.magnitude,
    });
  }

  // Stop any current flinch return animations
  if (currentFlinchReturn) {
    currentFlinchReturn.cancel();
    currentFlinchReturn = undefined;
  }

  injectParameterData(socket, {
    faceFound: false,
    mode: "add",
    parameterValues,
  });

  currentFlinchReturn = createFlinchReturn(socket, modelParameters, config);
}

/**
 * Creates an animation for returning from a flinch
 *
 * @param socket Socket for sending impact flinches to VTube studio
 * @param modelParameters Parameters for the current model
 * @param config Configuration for flinching
 * @returns Cancellable flinch return object to manage the current flinch
 */
function createFlinchReturn(
  socket: VTubeStudioWebSocket,
  modelParameters: ModelParameters,
  config: FlinchConfig,
): FlinchReturn {
  let running = true;
  let flinchWeight: number = 1;
  let lastTimestamp: DOMHighResTimeStamp = -1;

  function animateReturn(timestamp: DOMHighResTimeStamp) {
    // Don't schedule next animation frame, we are done
    if (!running || flinchWeight <= 0) {
      return;
    }

    // Initial animation pass
    if (lastTimestamp === -1) {
      lastTimestamp = timestamp;
    }

    // Run animation if enough time has elapsed
    if (timestamp - lastTimestamp >= FLINCH_RETURN_ANIMATE_INTERVAL) {
      flinchWeight -= 1 / config.returnSpeed / 60.0;
      if (flinchWeight <= 0) flinchWeight = 0;

      flinchReturn(socket, modelParameters, config, flinchWeight);
    }

    // Schedule next animation frame
    requestAnimationFrame(animateReturn);
  }

  requestAnimationFrame(animateReturn);

  // Cancel the flinch return
  const cancel = () => {
    running = false;
  };

  return {
    cancel,
  };
}

export function flinchReturn(
  socket: VTubeStudioWebSocket,
  modelParameters: ModelParameters,
  config: FlinchConfig,
  flinchWeight: number,
) {
  const parameterValues: InjectParameterValue[] = [];

  for (const param of modelParameters.horizontal) {
    const value = config.leftSide ? param.max : param.min;
    parameterValues.push({
      id: param.name,
      value: value * config.magnitude * flinchWeight,
    });
  }

  for (const param of modelParameters.vertical) {
    let value = config.angle > 0 ? param.min : param.max;
    value = (value * Math.abs(config.angle)) / 45;

    parameterValues.push({
      id: param.name,
      value: value * config.magnitude * flinchWeight,
    });
  }

  if (config.eyeState !== EyesMode.Unchanged) {
    for (const param of modelParameters.eyes) {
      let value =
        (config.leftSide ? param.max : param.min) *
        config.magnitude *
        flinchWeight;

      if (config.eyeState === EyesMode.Closed) value = -value;

      parameterValues.push({
        id: param.name,
        value,
      });
    }
  }

  injectParameterData(socket, {
    faceFound: false,
    mode: "add",
    parameterValues,
  });
}
