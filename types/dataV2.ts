import type { Uuid } from "./appData";

export type ItemId = Uuid;
export type SoundId = Uuid;

export type ItemImageConfig = {
  src: string;
  weight: number;
  scale: number;
  pixelate: boolean;
};

export type ItemWindupConfig = {
  enabled: boolean;
  duration: number;
};

export type ItemConfig = {
  image: ItemImageConfig;
  windup: ItemWindupConfig;
};
