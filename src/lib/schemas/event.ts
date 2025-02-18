import { z } from "zod";
import { minMax } from "$lib/utils/validation";
import {
  EventOutcomeType,
  EventTriggerType,
  ThrowableDataType,
  MinimumRequiredRole,
  MINIMUM_REQUIRED_ROLE_VALUES,
} from "$lib/api/types";

export const eventTriggerSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal(EventTriggerType.Redeem),
    reward_id: z.string(),
  }),
  z.object({
    type: z.literal(EventTriggerType.Command),
    message: z.string(),
  }),
  z.object({
    type: z.literal(EventTriggerType.Follow),
  }),
  z.object({
    type: z.literal(EventTriggerType.Subscription),
  }),
  z.object({
    type: z.literal(EventTriggerType.GiftedSubscription),
  }),
  z.object({
    type: z.literal(EventTriggerType.Bits),
    min_bits: z.number(),
  }),
  z.object({
    type: z.literal(EventTriggerType.Raid),
    min_raiders: z.number(),
  }),
  z.object({
    type: z.literal(EventTriggerType.Timer),
    interval: z.number(),
    min_chat_messages: z.number(),
  }),
  z.object({
    type: z.literal(EventTriggerType.AdBreakBegin),
  }),
  z.object({
    type: z.literal(EventTriggerType.ShoutoutReceive),
    min_viewers: z.number(),
  }),
]);

export type EventTriggerSchema = z.infer<typeof eventTriggerSchema>;

const inputAmountConfigSchema = z.object({
  multiplier: z.number(),
  range: minMax,
});

export const throwableDataSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal(ThrowableDataType.Throw),
    amount: z.number().default(1),
    use_input_amount: z.boolean().default(false),
    input_amount_config: inputAmountConfigSchema,
  }),
  z.object({
    type: z.literal(ThrowableDataType.Barrage),
    amount_per_throw: z.number(),
    frequency: z.number(),
    amount: z.number().default(1),
    use_input_amount: z.boolean().default(false),
    input_amount_config: inputAmountConfigSchema,
  }),
]);

export type ThrowableDataSchema = z.infer<typeof throwableDataSchema>;

export const eventOutcomeSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal(EventOutcomeType.ThrowBits),
    _1: z.string().nullable(),
    _100: z.string().nullable(),
    _1000: z.string().nullable(),
    _5000: z.string().nullable(),
    _10000: z.string().nullable(),
    amount: throwableDataSchema,
  }),
  z.object({
    type: z.literal(EventOutcomeType.Throwable),
    throwable_ids: z.array(z.string()),
    amount: throwableDataSchema,
  }),

  z.object({
    type: z.literal(EventOutcomeType.TriggerHotkey),
    hotkey_id: z.string(),
  }),
  z.object({
    type: z.literal(EventOutcomeType.PlaySound),
    sound_id: z.string(),
  }),
  z.object({
    type: z.literal(EventOutcomeType.SendChatMessage),
    template: z.string(),
  }),
  z.object({
    type: z.literal(EventOutcomeType.Script),
    script: z.string(),
  }),
  z.object({
    type: z.literal(EventOutcomeType.ChannelEmotes),
    amount: throwableDataSchema,
  }),
]);

export type EventOutcomeSchema = z.infer<typeof eventOutcomeSchema>;

const cooldownSchema = z.object({
  enabled: z.boolean(),
  duration: z.number(),
  per_user: z.boolean(),
});

const eventConfigSchema = z.object({
  trigger: eventTriggerSchema,
  outcome: eventOutcomeSchema,
  require_role: z.enum(MINIMUM_REQUIRED_ROLE_VALUES),
  cooldown: cooldownSchema,
  outcome_delay: z.number(),
});

export type EventConfigSchema = z.infer<typeof eventConfigSchema>;

export const eventSchema = z.object({
  name: z.string().min(1, "Name is required"),
  enabled: z.boolean(),
  config: eventConfigSchema,
});

export type EventSchema = z.infer<typeof eventSchema>;

export function getThrowableDataDefaults(): Record<
  ThrowableDataType,
  ThrowableDataSchema
> {
  return {
    [ThrowableDataType.Throw]: getThrowableDataDefault(ThrowableDataType.Throw),
    [ThrowableDataType.Barrage]: getThrowableDataDefault(
      ThrowableDataType.Barrage,
    ),
  };
}

export function getThrowableDataDefault(
  type: ThrowableDataType,
): ThrowableDataSchema {
  switch (type) {
    case ThrowableDataType.Throw:
      return {
        type: ThrowableDataType.Throw,
        amount: 1,
        use_input_amount: false,
        input_amount_config: {
          multiplier: 1,
          range: { min: 1, max: 1000 },
        },
      };
    case ThrowableDataType.Barrage:
      return {
        type: ThrowableDataType.Barrage,
        amount: 50,
        amount_per_throw: 5,
        frequency: 100,
        use_input_amount: false,
        input_amount_config: {
          multiplier: 1,
          range: { min: 1, max: 1000 },
        },
      };
  }
}

export function getEventOutcomeDefaults(): Record<
  EventOutcomeType,
  EventOutcomeSchema
> {
  return {
    [EventOutcomeType.ThrowBits]: getEventOutcomeDefault(
      EventOutcomeType.ThrowBits,
    ),
    [EventOutcomeType.Throwable]: getEventOutcomeDefault(
      EventOutcomeType.Throwable,
    ),
    [EventOutcomeType.TriggerHotkey]: getEventOutcomeDefault(
      EventOutcomeType.TriggerHotkey,
    ),
    [EventOutcomeType.PlaySound]: getEventOutcomeDefault(
      EventOutcomeType.PlaySound,
    ),
    [EventOutcomeType.SendChatMessage]: getEventOutcomeDefault(
      EventOutcomeType.SendChatMessage,
    ),
    [EventOutcomeType.Script]: getEventOutcomeDefault(EventOutcomeType.Script),
    [EventOutcomeType.ChannelEmotes]: getEventOutcomeDefault(
      EventOutcomeType.ChannelEmotes,
    ),
  };
}

export function getEventOutcomeDefault(
  type: EventOutcomeType,
): EventOutcomeSchema {
  switch (type) {
    case EventOutcomeType.ThrowBits:
      return {
        type: EventOutcomeType.ThrowBits,
        _1: null,
        _100: null,
        _1000: null,
        _10000: null,
        _5000: null,
        amount: {
          type: ThrowableDataType.Barrage,
          amount: 15,
          amount_per_throw: 5,
          frequency: 100,
          use_input_amount: false,
          input_amount_config: {
            multiplier: 1,
            range: { min: 1, max: 100 },
          },
        },
      };
    case EventOutcomeType.Throwable:
      return {
        type: EventOutcomeType.Throwable,
        throwable_ids: [],
        amount: {
          type: ThrowableDataType.Barrage,
          amount: 15,
          amount_per_throw: 5,
          frequency: 100,
          use_input_amount: false,
          input_amount_config: {
            multiplier: 1,
            range: { min: 1, max: 100 },
          },
        },
      };
    case EventOutcomeType.TriggerHotkey:
      return {
        type: EventOutcomeType.TriggerHotkey,
        hotkey_id: "",
      };
    case EventOutcomeType.PlaySound:
      return {
        type: EventOutcomeType.PlaySound,
        sound_id: "",
      };
    case EventOutcomeType.SendChatMessage:
      return {
        type: EventOutcomeType.SendChatMessage,
        template: "",
      };
    case EventOutcomeType.Script:
      return {
        type: EventOutcomeType.Script,
        script: "",
      };
    case EventOutcomeType.ChannelEmotes:
      return {
        type: EventOutcomeType.ChannelEmotes,
        amount: {
          type: ThrowableDataType.Barrage,
          amount: 15,
          amount_per_throw: 5,
          frequency: 100,
          use_input_amount: false,
          input_amount_config: {
            multiplier: 1,
            range: { min: 1, max: 100 },
          },
        },
      };
  }
}

export function getEventTriggerDefaults(): Record<
  EventTriggerType,
  EventTriggerSchema
> {
  return {
    [EventTriggerType.Redeem]: getEventTriggerDefault(EventTriggerType.Redeem),
    [EventTriggerType.Command]: getEventTriggerDefault(
      EventTriggerType.Command,
    ),
    [EventTriggerType.Follow]: getEventTriggerDefault(EventTriggerType.Follow),
    [EventTriggerType.Subscription]: getEventTriggerDefault(
      EventTriggerType.Subscription,
    ),
    [EventTriggerType.GiftedSubscription]: getEventTriggerDefault(
      EventTriggerType.GiftedSubscription,
    ),
    [EventTriggerType.Bits]: getEventTriggerDefault(EventTriggerType.Bits),
    [EventTriggerType.Raid]: getEventTriggerDefault(EventTriggerType.Raid),
    [EventTriggerType.Timer]: getEventTriggerDefault(EventTriggerType.Timer),
    [EventTriggerType.AdBreakBegin]: getEventTriggerDefault(
      EventTriggerType.AdBreakBegin,
    ),
    [EventTriggerType.ShoutoutReceive]: getEventTriggerDefault(
      EventTriggerType.ShoutoutReceive,
    ),
  };
}

export function getEventTriggerDefault(
  type: EventTriggerType,
): EventTriggerSchema {
  switch (type) {
    case EventTriggerType.Redeem:
      return { type: EventTriggerType.Redeem, reward_id: "" };
    case EventTriggerType.Command:
      return { type: EventTriggerType.Command, message: "!test" };
    case EventTriggerType.Follow:
      return { type: EventTriggerType.Follow };
    case EventTriggerType.Subscription:
      return { type: EventTriggerType.Subscription };
    case EventTriggerType.GiftedSubscription:
      return { type: EventTriggerType.GiftedSubscription };
    case EventTriggerType.Bits:
      return { type: EventTriggerType.Bits, min_bits: 1 };
    case EventTriggerType.Raid:
      return { type: EventTriggerType.Raid, min_raiders: 1 };
    case EventTriggerType.Timer:
      return {
        type: EventTriggerType.Timer,
        interval: 60,
        min_chat_messages: 2,
      };
    case EventTriggerType.AdBreakBegin:
      return { type: EventTriggerType.AdBreakBegin };
    case EventTriggerType.ShoutoutReceive:
      return { type: EventTriggerType.ShoutoutReceive, min_viewers: 1 };
  }
}

export function getDefaultEvent(): EventSchema {
  return {
    name: "",
    enabled: true,
    config: {
      trigger: getEventTriggerDefault(EventTriggerType.Redeem),
      outcome: getEventOutcomeDefault(EventOutcomeType.Throwable),
      require_role: MinimumRequiredRole.None,
      cooldown: { enabled: false, duration: 0, per_user: false },
      outcome_delay: 0,
    },
  };
}

// List of event triggers that provide an input value
const EVENT_TRIGGERS_WITH_INPUT = [
  EventTriggerType.Bits,
  EventTriggerType.GiftedSubscription,
  EventTriggerType.Subscription,
  EventTriggerType.Raid,
] as const;

export type EventTriggerTypeWithInput =
  (typeof EVENT_TRIGGERS_WITH_INPUT)[number];

export function isEventTriggerWithInput(
  type: EventTriggerType,
): type is EventTriggerTypeWithInput {
  return (EVENT_TRIGGERS_WITH_INPUT as readonly EventTriggerType[]).includes(
    type,
  );
}
