import { ItemId, SoundId, ItemConfig } from "$shared/dataV2";

export * from "$shared/dataV2";
export * from "$shared/appData";
export * from "$shared/runtimeAppData";

export const enum ThrowItemConfigType {
  Barrage = "Barrage",
  All = "All",
}

/**
 * Sound with UI specific fields stripped
 */
export type PartialSoundModel = {
  id: SoundId;
  src: string;
  volume: number;
};

export type ThrowItemConfig =
  | { type: ThrowItemConfigType.All; amount: number }
  | {
      type: ThrowItemConfigType.Barrage;
      amount_per_throw: number;
      amount: number;
      frequency: number;
    };

export type PartialItemModel = {
  id: ItemId;
  config: ItemConfig;
};

export type ItemWithSoundIds = PartialItemModel & {
  impact_sound_ids: SoundId[];
  windup_sound_ids: SoundId[];
};

export type ItemWithSounds = {
  items: ItemWithSoundIds[];
  sounds: PartialSoundModel[];
};
