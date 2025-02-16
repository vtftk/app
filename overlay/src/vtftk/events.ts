import { updateRuntimeData } from "./api";
import { BACKEND_HTTP } from "../constants";
import { beginCalibrationStep } from "./calibration";
import { CalibrationStep } from "./calibration-types";
import { VTubeStudioWebSocket } from "../vtube-studio/socket";
import { triggerHotkey, requestHotkeys } from "../vtube-studio/hotkeys";
import { ModelParameters, requestMoveModel } from "../vtube-studio/model";
import { throwItem, setPhysicsEngineConfig } from "../vtube-studio/throw-item";
import {
  loadAudio,
  loadItems,
  loadSounds,
  arrayRandom,
  LoadedItemMap,
  LoadedSoundMap,
  executeInterval,
} from "../utils";
import {
  ModelId,
  OverlayConfig,
  ItemWithSounds,
  ThrowItemConfig,
  ModelCalibration,
  ItemWithSoundIds,
  PartialSoundModel,
  ThrowItemConfigType,
} from "./types";

export type EventSourceData = {
  overlayConfig: OverlayConfig;
  vtSocket: VTubeStudioWebSocket | undefined;
  modelCalibration: Map<ModelId, ModelCalibration>;
  modelParameters: ModelParameters | undefined;
};

export function createEventSource(data: EventSourceData) {
  const eventSource = new EventSource(new URL("/overlay/events", BACKEND_HTTP));

  eventSource.addEventListener("open", () => {
    console.debug("listening to events");
  });

  eventSource.addEventListener("message", (event) => {
    const eventData = JSON.parse(event.data);
    onMessage(data, eventData);
  });

  eventSource.addEventListener("error", (event) => {
    console.error("event source error", event);
  });

  return eventSource;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function onMessage(data: EventSourceData, event: any) {
  switch (event.type) {
    case "ThrowItem": {
      if (data.vtSocket && data.modelParameters) {
        onThrowItemEvent(
          data.overlayConfig,
          data.vtSocket,
          data.modelCalibration,
          data.modelParameters,
          event.items,
          event.config,
        );
      }

      break;
    }

    case "TriggerHotkey": {
      if (data.vtSocket) {
        onTriggerHotkeyEvent(data.vtSocket, event.hotkey_id);
      }

      break;
    }

    case "TriggerHotkeyByName": {
      if (data.vtSocket) {
        onTriggerHotkeyByNameEvent(
          data.vtSocket,
          event.hotkey_name,
          event.ignore_case,
        );
      }

      break;
    }

    case "PlaySound": {
      if (data.vtSocket) {
        onPlaySoundEvent(data.overlayConfig, event.config);
      }

      break;
    }

    case "PlaySoundSeq": {
      if (data.vtSocket) {
        onPlaySoundSeqEvent(data.overlayConfig, event.configs);
      }

      break;
    }

    case "ConfigUpdated": {
      onConfigUpdatedEvent(data, event.config);
      break;
    }

    case "UpdateHotkeys": {
      if (data.vtSocket) {
        onUpdateHotkeysEvent(data.vtSocket);
      }

      break;
    }

    case "SetCalibrationStep": {
      if (data.vtSocket) {
        onSetCalibrationStepEvent(data, data.vtSocket, event.step);
      }
      break;
    }

    case "MoveModel": {
      if (data.vtSocket) {
        onMoveModelEvent(data.vtSocket, event.x, event.y);
      }
      break;
    }
  }
}

function onConfigUpdatedEvent(
  data: EventSourceData,
  overlayConfig: OverlayConfig,
) {
  data.overlayConfig = overlayConfig;

  // Recreate the physics engine
  const { fps, gravity_multiplier } = overlayConfig.physics_config;
  setPhysicsEngineConfig({
    fps: fps,
    gravityMultiplier: gravity_multiplier,
  });
}

async function onUpdateHotkeysEvent(vtSocket: VTubeStudioWebSocket) {
  const hotkeys = await requestHotkeys(vtSocket);

  await updateRuntimeData({
    hotkeys: hotkeys.map((hotkey) => ({
      hotkey_id: hotkey.hotkeyID,
      name: hotkey.name,
    })),
  });

  return hotkeys;
}

async function onPlaySoundEvent(
  overlayConfig: OverlayConfig,
  config: PartialSoundModel,
) {
  const audio = await loadAudio(config.src);
  audio.volume = config.volume * overlayConfig.sounds_config.global_volume;
  audio.play();
}

async function onPlaySoundSeqEvent(
  appData: OverlayConfig,
  configs: PartialSoundModel[],
) {
  const sounds = await loadSounds(configs);

  for (const config of configs) {
    console.debug("Playing sound config", config);

    const soundData = sounds.get(config.id);

    if (soundData === undefined) {
      console.warn("Skipping sound config that failed to load", config);
      continue;
    }

    // Play the sound
    const audio = soundData.sound;
    audio.volume = config.volume * appData.sounds_config.global_volume;

    const completePromise = new Promise<void>((resolve, reject) => {
      audio.onended = () => resolve();
      audio.onerror = () => reject();
    });

    audio.play();

    // Wait for the sound to complete fully
    await completePromise;

    console.debug("Completed sound config", config);
  }
}

async function onTriggerHotkeyEvent(
  vtSocket: VTubeStudioWebSocket,
  hotkeyID: string,
) {
  const hotkeys = await triggerHotkey(vtSocket, hotkeyID);

  return hotkeys;
}

async function onTriggerHotkeyByNameEvent(
  vtSocket: VTubeStudioWebSocket,
  hotkeyName: string,
  ignoreCase: boolean,
) {
  const hotkeys = await requestHotkeys(vtSocket);

  const hotkey = hotkeys.find((hotkey) => {
    if (ignoreCase) {
      return hotkey.name.toLowerCase() === hotkeyName.toLowerCase();
    } else {
      return hotkey.name === hotkeyName;
    }
  });

  if (hotkey === undefined) return;

  await triggerHotkey(vtSocket, hotkey.hotkeyID);
}

async function onSetCalibrationStepEvent(
  data: EventSourceData,
  vtSocket: VTubeStudioWebSocket,
  step: CalibrationStep,
) {
  beginCalibrationStep(vtSocket, step, (model_data) => {
    // Update the model data map to include the new model data
    data.modelCalibration.set(model_data.id, model_data.calibration);
  });
}
async function onMoveModelEvent(
  vtSocket: VTubeStudioWebSocket,
  x: number,
  y: number,
) {
  await requestMoveModel(vtSocket, {
    valuesAreRelativeToModel: true,
    timeInSeconds: 1,
    positionX: x,
    positionY: y,
  });
}

async function onThrowItemEvent(
  overlayConfig: OverlayConfig,
  vtSocket: VTubeStudioWebSocket,
  modelCalibration: Map<ModelId, ModelCalibration>,
  modelParameters: ModelParameters,
  items: ItemWithSounds,
  config: ThrowItemConfig,
) {
  const [loadedItems, loadedSounds] = await Promise.all([
    loadItems(items.items),
    loadSounds(items.sounds),
  ]);

  if (config.type === ThrowItemConfigType.All) {
    await throwItemMany(
      vtSocket,
      overlayConfig,
      modelCalibration,
      modelParameters,
      items.items,
      loadedItems,
      loadedSounds,
      config.amount,
    );
  } else if (config.type === ThrowItemConfigType.Barrage) {
    await executeInterval(
      async () => {
        return throwItemMany(
          vtSocket,
          overlayConfig,
          modelCalibration,
          modelParameters,
          items.items,
          loadedItems,
          loadedSounds,
          config.amount_per_throw,
        );
      },
      config.frequency,
      config.amount,
    );
  }
}

function pickRandomSound(
  soundIds: string[],
  sounds: LoadedSoundMap,
  clone: boolean = false,
) {
  if (soundIds.length < 1) return null;

  const randomSoundId = arrayRandom(soundIds);
  const audio = sounds.get(randomSoundId);

  if (audio) {
    return {
      config: audio.config,
      sound: clone
        ? (audio.sound.cloneNode() as HTMLAudioElement)
        : audio.sound,
    };
  }

  return null;
}

function pickRandomItem(items: ItemWithSoundIds[], images: LoadedItemMap) {
  if (items.length === 1) {
    const item = items[0];
    const image = images.get(item.id);
    if (image === undefined) return null;
    return { config: item, image };
  }

  if (items.length > 0) {
    const randomItem = arrayRandom(items);
    const image = images.get(randomItem.id);
    if (image !== undefined) {
      return {
        config: randomItem,
        image,
      };
    }
  }

  return null;
}

function throwRandomItem(
  socket: VTubeStudioWebSocket,
  overlayConfig: OverlayConfig,
  modelCalibration: Map<ModelId, ModelCalibration>,
  modelParameters: ModelParameters,

  items: ItemWithSoundIds[],
  loadedItems: LoadedItemMap,
  loadedSounds: LoadedSoundMap,
): Promise<void> {
  const item = pickRandomItem(items, loadedItems);

  // No item found
  if (item === null) return Promise.resolve();

  const impactAudio = pickRandomSound(
    item.config.impact_sound_ids,
    loadedSounds,
  );

  const windupAudio = pickRandomSound(
    item.config.windup_sound_ids,
    loadedSounds,
  );

  return throwItem(
    socket,
    overlayConfig,
    modelCalibration,

    modelParameters,
    item.config,
    item.image,
    impactAudio,
    windupAudio,
  );
}

async function throwItemMany(
  socket: VTubeStudioWebSocket,
  overlayConfig: OverlayConfig,
  modelCalibration: Map<ModelId, ModelCalibration>,
  modelParameters: ModelParameters,

  items: ItemWithSoundIds[],
  loadedItems: LoadedItemMap,
  loadedSounds: LoadedSoundMap,
  amount: number,
) {
  if (amount === 1) {
    return throwRandomItem(
      socket,
      overlayConfig,
      modelCalibration,
      modelParameters,
      items,
      loadedItems,
      loadedSounds,
    );
  }

  return Promise.all(
    Array.from(Array(amount)).map(() =>
      throwRandomItem(
        socket,
        overlayConfig,
        modelCalibration,
        modelParameters,
        items,
        loadedItems,
        loadedSounds,
      ),
    ),
  );
}
