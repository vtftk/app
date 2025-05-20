export * from "$shared/dataV2";
export * from "$shared/appData";
export * from "$shared/runtimeAppData";

import type { Uuid, MinMax } from "$shared/appData";
import type { ItemId, SoundId, ItemConfig } from "$shared/dataV2";

type Option<T> = T | null;

export type Sound = {
  id: SoundId;
  name: string;
  src: string;
  volume: number;
  order: number;
};

export type Item = {
  id: ItemId;
  name: string;
  config: ItemConfig;
  order: number;
};

export type ItemWithImpactSounds = Item & {
  impact_sounds_ids: SoundId[];
  windup_sounds_ids: SoundId[];
};

// File types for file uploads
export enum StorageFolder {
  ThrowableImage = "ThrowableImage",
  ImpactSound = "ImpactSound",
  ImpactImage = "ImpactImage",
  Sound = "Sound",
}

export type VTubeStudioBroadcast = {
  apiName: string;
  apiVersion: string;
  timestamp: number;
  messageType: string;
  requestID: string;
  data: {
    active: boolean;
    port: number;
    instanceID: number;
    windowTitle: string;
  };
};

export type CommandId = Uuid;

export type Command = {
  id: Uuid;
  enabled: boolean;
  name: string;
  command: string;
  config: CommandConfig;
  order: number;
};

export type CommandWithAliases = Command & {
  aliases: string[];
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
  config: CommandConfig;
  aliases: string[];
};

export type CommandConfig = {
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
    config: CommandConfig;
    order: number;
    aliases: string[];
  }>;
};

export type CommandExecution = {
  command_id: string;
} & ExecutionData;

export type EventId = Uuid;

export type VEvent = {
  id: EventId;
  name: string;
  enabled: boolean;
  config: EventConfig;
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
  config: EventConfig;
};

export type EventConfig = {
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
    config: EventConfig;
    order: number;
  }>;
};

export type VEventData = {
  user: {
    id: string;
    name: string;
    displayName: string;
  } | null;
} & EventInputData;

interface RedeemInputData {
  /**
   * Unique ID for the specific redemption event
   */
  redemptionId: string;
  /**
   * Name of the redeem
   */
  rewardName: string;
  /**
   * Unique ID for the redeem
   */
  rewardId: string;
  /**
   * Channel points cost for the redeem
   */
  cost: number;
  /**
   * User provided message if the redeem
   * requests a message
   */
  userInput: string;
}

interface BitsInputData {
  /**
   * Number of bits gifted
   */
  bits: number;
  /**
   * Whether the bits were gifted anonymously
   */
  anonymous: boolean;
  /**
   * Message provided with the gifted bits
   */
  message: string;
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
interface FollowInputData {}

interface SubscriptionInputData {
  /**
   * Tier of the subscription
   */
  tier: SubscriptionTier;
  /**
   * Whether the subscription was gifted
   */
  isGift: boolean;
}

interface GiftedSubscriptionInputData {
  /**
   * Tier of subscription gifted
   */
  tier: SubscriptionTier;
  /**
   * The number of subscriptions gifted by this user in the channel.
   * Null when anonymous gifter
   */
  cumulativeTotal: number | null;
  /**
   * Whether it was gifted anonymously
   */
  anonymous: boolean;
  /**
   * Total number of subs gifted
   */
  total: number;
}

interface ResubscriptionInputData {
  /**
   * The total number of months the user has been subscribed to the channel.
   */
  cumulativeMonths: number;
  /**
   * The month duration of the subscription. (The gifted amount)
   */
  durationMonths: number;
  /**
   * User message attached to the resubscription
   */
  message: string;
  /**
   * The number of consecutive months the user’s current subscription has been active.
   * This value is null if the user has opted out of sharing this information.
   */
  streakMonths: number | null;
  /**
   * Tier resubscribed at
   */
  tier: SubscriptionTier;
}

interface ChatInputData {
  /**
   * ID of the chat message
   */
  messageId: string;
  /**
   * The chat message content
   */
  message: string;
  /**
   * Individual chat fragments
   */
  fragments: ChatFragment[];
  /**
   * Cheered bits amount if the message contained a bits cheer
   */
  cheer: number | null;
}

interface Cheermote {
  /**
   * The name portion of the Cheermote string that you use in chat to cheer Bits.
   *
   * The full Cheermote string is the concatenation of {prefix} + {number of Bits}.
   * For example, if the prefix is “Cheer” and you want to cheer 100 Bits, the full Cheermote string is Cheer100.
   * When the Cheermote string is entered in chat, Twitch converts it to the image associated with the Bits tier that was cheered.
   */
  prefix: string;
  /**
   * The amount of bits cheered.
   */
  bits: number;
  /**
   * The tier level of the cheermote.
   */
  tier: number;
}

interface Emote {
  /**
   * An ID that uniquely identifies this emote.
   */
  id: string;
  /**
   * An ID that identifies the emote set that the emote belongs to.
   */
  emote_set_id: string;
  /**
   * The ID of the broadcaster who owns the emote.
   */
  owner_id: string;
  /**
   * The formats that the emote is available in. For example, if the emote is available only as a static PNG, the array contains only static. But if the emote is available as a static PNG and an animated GIF, the array contains static and animated. The possible formats are:
   * * `animated` — An animated GIF is available for this emote.
   * * `static` — A static PNG file is available for this emote.
   */
  format: ("animated" | "static")[];
}

interface Mention {
  /**
   * The user ID of the mentioned user. (Unique ID)
   */
  user_id: string;
  /**
   * The user name of the mentioned user. (Display name)
   */
  user_name: string;
  /**
   * The user login of the mentioned user. (Username)
   */
  user_login: string;
}

type ChatFragment =
  | {
      type: "Cheermote";
      text: string;
      cheermote: Cheermote;
    }
  | { type: "Emote"; text: string; emote: Emote }
  | { type: "Mention"; text: string; mention: Mention }
  | { type: "Text"; text: string };

interface RaidInputData {
  /**
   * Number of viewers present in the raid
   */
  viewers: number;
}

interface AdBreakBeginInputData {
  /**
   * Duration in seconds for the ad break that is starting
   */
  durationSeconds: number;
}

interface ShoutoutReceiveInputData {
  /**
   * Number of viewers that have seen the shoutout
   */
  viewerCount: number;
}

export type EventInputData =
  | RedeemInputData
  | BitsInputData
  | FollowInputData
  | SubscriptionInputData
  | GiftedSubscriptionInputData
  | ResubscriptionInputData
  | ChatInputData
  | RaidInputData
  | AdBreakBeginInputData
  | ShoutoutReceiveInputData;

export enum SubscriptionTier {
  Tier1 = "1000",
  Tier2 = "2000",
  Tier3 = "3000",
  Prime = "Prime",
}

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
    displayName: string;
  } | null;
  input_data: Partial<EventInputData>;
}>;

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

export type CreateItem = {
  name: string;
  config: ItemConfig;
  impact_sounds: SoundId[];
  windup_sounds: SoundId[];
};

export type UpdateItem = {
  itemId: ItemId;
  update: Partial<{
    name: string;
    config: ItemConfig;
    impact_sounds: SoundId[];
    windup_sounds: SoundId[];
  }>;
};

export type UpdateOrdering = {
  id: string;
  order: number;
};

export enum MinimumRequiredRole {
  None = "None",
  Follower = "Follower",
  Vip = "Vip",
  Mod = "Mod",
  Broadcaster = "Broadcaster",
}

export const MINIMUM_REQUIRED_ROLE_VALUES = [
  MinimumRequiredRole.None,
  MinimumRequiredRole.Follower,
  MinimumRequiredRole.Vip,
  MinimumRequiredRole.Mod,
  MinimumRequiredRole.Broadcaster,
] as const;

export enum EventTriggerType {
  Redeem = "Redeem",
  Command = "Command",
  Follow = "Follow",
  Subscription = "Subscription",
  GiftedSubscription = "GiftedSubscription",
  Bits = "Bits",
  Raid = "Raid",
  Timer = "Timer",
  AdBreakBegin = "AdBreakBegin",
  ShoutoutReceive = "ShoutoutReceive",
}

export type EventTrigger =
  | {
      type: EventTriggerType.Redeem;
      reward_id: string;
    }
  | { type: EventTriggerType.Command; message: string }
  | { type: EventTriggerType.Follow }
  | { type: EventTriggerType.Subscription }
  | { type: EventTriggerType.GiftedSubscription }
  | { type: EventTriggerType.Bits; min_bits: number }
  | {
      type: EventTriggerType.Raid;
      min_raiders: number;
    }
  | {
      type: EventTriggerType.Timer;
      interval: number;
      min_chat_messages: number;
    }
  | {
      type: EventTriggerType.AdBreakBegin;
    }
  | {
      type: EventTriggerType.ShoutoutReceive;
      min_viewers: number;
    };

export enum ThrowableDataType {
  Throw = "Throw",
  Barrage = "Barrage",
}

export type ThrowableDataThrow = {
  amount: number;
  use_input_amount: boolean;
  input_amount_config: InputAmountConfig;
};

export type ThrowableDataBarrage = {
  amount_per_throw: number;
  frequency: number;
  amount: number;
  use_input_amount: boolean;
  input_amount_config: InputAmountConfig;
};

export type InputAmountConfig = {
  multiplier: number;
  range: MinMax;
};

export type ThrowableData =
  | ({ type: ThrowableDataType.Throw } & ThrowableDataThrow)
  | ({ type: ThrowableDataType.Barrage } & ThrowableDataBarrage);

export enum BitsAmountType {
  Fixed = "Fixed",
  Dynamic = "Dynamic",
}

export type BitsAmountFixed = {
  amount: number;
};

export type BitsAmountDynamic = {
  max_amount: number;
};

export enum EventOutcomeType {
  ThrowBits = "ThrowBits",
  Throwable = "Throwable",
  TriggerHotkey = "TriggerHotkey",
  PlaySound = "PlaySound",
  SendChatMessage = "SendChatMessage",
  Script = "Script",
  ChannelEmotes = "ChannelEmotes",
}

export type EventOutcomeBits = {
  _1: Option<Uuid>;
  _100: Option<Uuid>;
  _1000: Option<Uuid>;
  _5000: Option<Uuid>;
  _10000: Option<Uuid>;
  amount: ThrowableData;
};
export type EventOutcomeThrowable = {
  throwable_ids: Uuid[];
  amount: ThrowableData;
};
export type EventOutcomeTriggerHotkey = { hotkey_id: Uuid };
export type EventOutcomePlaySound = { sound_id: Uuid };
export type EventOutcomeSendChatMessage = { template: string };
export type EventOutcomeScript = { script: string };
export type EventOutcomeChannelEmotes = {
  amount: ThrowableData;
};
export type EventOutcome =
  | ({ type: EventOutcomeType.ThrowBits } & EventOutcomeBits)
  | ({ type: EventOutcomeType.Throwable } & EventOutcomeThrowable)
  | ({ type: EventOutcomeType.TriggerHotkey } & EventOutcomeTriggerHotkey)
  | ({ type: EventOutcomeType.PlaySound } & EventOutcomePlaySound)
  | ({ type: EventOutcomeType.SendChatMessage } & EventOutcomeSendChatMessage)
  | ({ type: EventOutcomeType.Script } & EventOutcomeScript)
  | ({ type: EventOutcomeType.ChannelEmotes } & EventOutcomeChannelEmotes);

export type EventOutcomeVariant<T extends EventOutcomeType> = Extract<
  EventOutcome,
  { type: T }
>;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type CustomReward = any;

export enum CommandOutcomeType {
  Template = "Template",
  Script = "Script",
}

export type CommandOutcomeTemplate = { message: string };
export type CommandOutcomeScript = { script: string };

export type CommandOutcome =
  | ({ type: CommandOutcomeType.Template } & CommandOutcomeTemplate)
  | ({ type: CommandOutcomeType.Script } & CommandOutcomeScript);
