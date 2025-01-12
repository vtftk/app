import type {
  Uuid,
  EventOutcome,
  EventTrigger,
  CommandOutcome,
  MinimumRequiredRole,
  ThrowableImageConfig,
} from "./appData";

export type ItemId = Uuid;

export type Item = {
  id: ItemId;
  name: string;
  image: ThrowableImageConfig;
  order: number;
};

export type UpdateOrdering = {
  id: string;
  order: number;
};

export type ItemWithImpactSoundIds = Item & {
  impact_sound_ids: SoundId[];
};

export type ItemWithImpactSounds = Item & { impact_sounds: Sound[] };

export type CreateItem = {
  name: string;
  image: ThrowableImageConfig;
  impact_sounds: SoundId[];
};

export type UpdateItem = {
  itemId: ItemId;
  update: Partial<{
    name: string;
    image: ThrowableImageConfig;
    impact_sounds: SoundId[];
    order: number;
  }>;
};

export type ItemWithSounds = {
  items: ItemWithImpactSoundIds[];
  impact_sounds: Sound[];
};

export const enum ThrowItemConfigType {
  Barrage = "Barrage",
  All = "All",
}

export type ThrowItemConfig =
  | { type: ThrowItemConfigType.All; amount: number }
  | {
      type: ThrowItemConfigType.Barrage;
      amount_per_throw: number;
      amount: number;
      frequency: number;
    };

export type SoundId = Uuid;

export type Sound = {
  id: SoundId;
  name: string;
  src: string;
  volume: number;
  order: number;
};

export type CreateSound = {
  name: string;
  src: string;
  volume: number;
};

export type UpdateSound = {
  soundId: SoundId;
  update: Partial<{
    name: string;
    src: string;
    volume: number;
    order: number;
  }>;
};

export type CommandId = Uuid;

export type Command = {
  id: Uuid;
  enabled: boolean;
  name: string;
  command: string;
  aliases: string[];
  outcome: CommandOutcome;
  cooldown: CommandCooldown;
  require_role: MinimumRequiredRole;
  order: number;
};

export type CommandCooldown = {
  enabled: boolean;
  duration: number;
  per_user: boolean;
};

export type CreateCommand = {
  enabled: boolean;
  name: string;
  command: string;
  aliases: string[];
  outcome: CommandOutcome;
  cooldown: CommandCooldown;
  require_role: MinimumRequiredRole;
};

export type UpdateCommand = {
  commandId: CommandId;
  update: Partial<{
    enabled: boolean;
    name: string;
    command: string;
    aliases: string[];
    outcome: CommandOutcome;
    cooldown: CommandCooldown;
    require_role: MinimumRequiredRole;
    order: number;
  }>;
};

export type EventId = Uuid;

export type VEvent = {
  id: EventId;
  name: string;
  enabled: boolean;
  trigger: EventTrigger;
  outcome: EventOutcome;
  cooldown: VEventCooldown;
  require_role: MinimumRequiredRole;
  outcome_delay: number;
  order: number;
};

export type VEventCooldown = {
  enabled: boolean;
  duration: number;
  per_user: boolean;
};

export type CreateEvent = {
  name: string;
  enabled: boolean;
  trigger: EventTrigger;
  outcome: EventOutcome;
  cooldown: VEventCooldown;
  require_role: MinimumRequiredRole;
  outcome_delay: number;
};

export type UpdateEvent = {
  eventId: EventId;
  update: Partial<{
    name: string;
    enabled: boolean;
    trigger: EventTrigger;
    outcome: EventOutcome;
    cooldown: VEventCooldown;
    require_role: MinimumRequiredRole;
    outcome_delay: number;
    order: number;
  }>;
};

export type VEventData = {
  user: {
    id: string;
    name: string;
    display_name: string;
  } | null;
} & EventInputData;

export type EventInputData =
  | EventInputDataNone
  | EventInputDataRedeem
  | EventInputDataBits
  | EventInputDataSubscription
  | EventInputDataGiftedSubscription
  | EventInputDataReSubscription
  | EventInputDataChat
  | EventInputDataRaid;

export type EventInputDataNone = object;
export type EventInputDataRedeem = {
  redemption_id: string;
  reward_id: string;
  reward_name: string;
  cost: number;
  user_input: string;
};
export type EventInputDataBits = {
  bits: number;
  anonymous: boolean;
  message: string;
};

export enum SubscriptionTier {
  Tier1 = "1000",
  Tier2 = "2000",
  Tier3 = "3000",
  Prime = "Prime",
}

export type EventInputDataSubscription = {
  tier: SubscriptionTier;
  is_gift: boolean;
};
export type EventInputDataGiftedSubscription = {
  tier: SubscriptionTier;
  cumulative_total: number | null;
  anonymous: boolean;
  total: number;
};
export type EventInputDataReSubscription = {
  cumulative_months: number;
  duration_months: number;
  message: string;
  streak_months: number | null;
  tier: SubscriptionTier;
};
export type EventInputDataChat = {
  message: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  fragments: any[];
  cheer: number | null;
};

export type EventInputDataRaid = {
  viewers: number;
};

export type EventLog = {
  event_id: string;
} & LogData;

export type CommandLog = {
  command_id: string;
} & LogData;

export type LogId = string;

export type LogData = {
  id: LogId;
  level: LoggingLevelStr;
  message: string;
  created_at: string;
};

export enum LoggingLevelDb {
  Debug = 0,
  Info = 1,
  Warn = 2,
  Error = 3,
}

export enum LoggingLevelStr {
  Debug = "Debug",
  Info = "Info",
  Warn = "Warn",
  Error = "Error",
}

export type LogsQuery = Partial<{
  level: LoggingLevelDb;
  start_date: string;
  end_date: string;
  offset: number;
  limit: number;
}>;

export type ExecutionsQuery = Partial<{
  start_date: string;
  end_date: string;
  offset: number;
  limit: number;
}>;

export type CommandExecution = {
  command_id: string;
} & ExecutionData;

export type EventExecution = {
  command_id: string;
} & ExecutionData;

export type ExecutionId = string;

export type ExecutionData = {
  id: ExecutionId;
  // Metadata could be anything
  metadata: UnstableExecutionMetadata;
  created_at: string;
};

// Type is unstable, can change at any time all fields
// must be checked and may not exist
export type UnstableExecutionMetadata = Partial<{
  user: {
    id: string;
    name: string;
    display_name: string;
  } | null;
  input_data: Partial<EventInputData>;
}>;
