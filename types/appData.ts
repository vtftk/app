export type Uuid = string;

export type MinMax = {
  min: number;
  max: number;
};

export type AppData = AppConfig & OverlayConfig;

export type AppConfig = {
  main_config: MainConfig;
};

export type OverlayConfig = {
  throwables_config: ThrowablesConfig;
  sounds_config: SoundsConfig;
  model_config: ModelConfig;
  vtube_studio_config: VTubeStudioConfig;
  physics_config: PhysicsConfig;
};

export type PhysicsConfig = {
  enabled: boolean;
  fps: number;
  gravity_multiplier: number;
  horizontal_multiplier: number;
  vertical_multiplier: number;
};

export type MainConfig = {
  minimize_to_tray: boolean;
  clean_logs: boolean;
  clean_logs_days: number;
  clean_executions: boolean;
  clean_executions_days: number;
  clean_chat_history: boolean;
  clean_chat_history_days: number;
  auto_updating: boolean;
  http_port: number;
};

export type ThrowablesConfig = {
  duration: number;
  spin_speed: MinMax;
  throw_angle: MinMax;
  direction: ThrowDirection;
  impact_delay: number;
  item_scale: MinMax;
};

export enum ThrowDirection {
  Weighted = "Weighted",
  Random = "Random",
  LeftOnly = "LeftOnly",
  RightOnly = "RightOnly",
}

export const THROW_DIRECTION_VALUES = [
  ThrowDirection.Weighted,
  ThrowDirection.Random,
  ThrowDirection.LeftOnly,
  ThrowDirection.RightOnly,
] as const;

export type ModelId = string;

export type ModelData = {
  id: ModelId;
  name: string;
  calibration: ModelCalibration;
};

export type ModelCalibration = {
  x: MinMax;
  y: MinMax;
};

export type SoundsConfig = {
  global_volume: number;
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

export const EYES_MODE_VALUES = [
  EyesMode.Unchanged,
  EyesMode.Opened,
  EyesMode.Closed,
] as const;

export type VTubeStudioConfig = {
  host: string;
  port: number;
};
