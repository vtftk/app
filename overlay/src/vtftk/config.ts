export type MinMax = {
  min: number;
  max: number;
};

export type AppData = {
  throwables: ThrowablesConfig;
  items: ItemsConfig;
  model: ModelConfig;
  models: Record<ModelId, ModelData>;
};

export type ThrowablesConfig = {
  duration: number;
  spin_speed: MinMax;
  throw_angle: MinMax;
  direction: ThrowDirection;
  impact_delay: number;
};

export enum ThrowDirection {
  Random = "Random",
  LeftOnly = "LeftOnly",
  RightOnly = "RightOnly",
}

export type ModelId = string;

export type ModelData = {
  x: MinMax;
  y: MinMax;
};

export type ItemsConfig = {
  global_volume: number;
  item_scale: MinMax;
};

export type ModelConfig = {
  model_return_time: number;
  eyes_on_hit: EyesMode;
};

export enum EyesMode {
  Unchanged = "Unchanged",
  Opened = "Opened",
  Closed = "Closed",
}
