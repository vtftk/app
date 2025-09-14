import getBackendURL from "./url";
import { PartialItemModel, PartialSoundModel } from "../vtftk/types";

export async function sleep(duration: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, duration));
}

/**
 * Runs a function on an interval
 *
 * @param action The action to run
 * @param interval The interval between each run
 * @param maxIterations The maximum number of times to run
 * @returns Promise for when its complete
 */
export async function executeInterval(
  action: VoidFunction,
  interval: number,
  maxIterations: number,
): Promise<void> {
  return new Promise((resolve) => {
    let iteration: number = 0;

    const intervalHandle = setInterval(() => {
      action();
      iteration += 1;

      if (iteration === maxIterations) {
        clearInterval(intervalHandle);
        resolve();
      }
    }, interval);
  });
}

export async function loadImage(src: string): Promise<HTMLImageElement> {
  const image = new Image();
  image.src = getBackendURL(src);

  return new Promise((resolve, reject) => {
    image.onload = () => resolve(image);
    image.onerror = (err) => reject(err);
  });
}

export async function loadAudio(src: string): Promise<HTMLAudioElement> {
  const audio = new Audio(getBackendURL(src));

  return new Promise((resolve, reject) => {
    audio.oncanplaythrough = () => resolve(audio);
    audio.onerror = () => reject();
  });
}

export type LoadedItemMap = Map<string, HTMLImageElement>;
export type LoadedSoundMap = Map<string, LoadedSoundData>;
export type LoadedSoundData = {
  config: PartialSoundModel;
  sound: HTMLAudioElement;
};

export async function loadItems(
  items: PartialItemModel[],
): Promise<LoadedItemMap> {
  const results = await Promise.allSettled(
    items.map(async (item) => ({
      id: item.id,
      image: await loadImage(item.config.image.src),
    })),
  );

  const output = new Map();

  for (const result of results) {
    if (result.status !== "fulfilled") {
      console.error(result);
      continue;
    }
    output.set(result.value.id, result.value.image);
  }

  return output;
}

export async function loadSounds(
  sounds: PartialSoundModel[],
): Promise<LoadedSoundMap> {
  const results = await Promise.allSettled(
    sounds.map(async (config) => ({
      sound: await loadAudio(config.src),
      config,
    })),
  );

  const output = new Map();

  for (const result of results) {
    if (result.status !== "fulfilled") {
      console.error(result);
      continue;
    }
    output.set(result.value.config.id, result.value);
  }

  return output;
}
