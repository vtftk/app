import { SoundsConfig } from "$shared/appData";

import { LoadedSoundData } from "../utils";

export default function playSound(
  soundData: LoadedSoundData,
  config: SoundsConfig,
) {
  try {
    const sound = soundData.sound;
    sound.volume = config.global_volume * soundData.config.volume;
    sound.play();
  } catch (err) {
    console.error("failed to play audio", err);
  }
}
