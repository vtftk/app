import {
  type VEventData,
  SubscriptionTier,
  EventTriggerType,
} from "$lib/api/types";

function randomNumber(min: number, max: number) {
  return Math.floor(Math.random() * (max - min)) + min;
}

function randomBool() {
  return Math.random() < 0.5;
}

function randomArrayItem<T>(values: T[]): T {
  return values[Math.floor(Math.random() * values.length)];
}

const BITS_MESSAGES = [
  "Here have some bits!",
  "Enjoy the test bits!",
  "Some bits for you!",
];

const REWARD_NAMES = ["Example Reward", "Twitch Redeem"];
const REWARD_MESSAGES = ["Example reward message!", "User input goes here"];

const USER_NAMES = ["test_user", "vtftk_user", "sample_user"];
const USER_DISPLAY_NAMES: Record<string, string> = {
  test_user: "TestUser",
  vtftk_user: "VTFTK User",
  sample_user: "Sample User",
};

export function getEventTestingData(triggerType: EventTriggerType): VEventData {
  const userName = randomArrayItem(USER_NAMES);
  const userDisplayname = USER_DISPLAY_NAMES[userName];
  const user = {
    id: "12826",
    name: userName,
    displayName: userDisplayname,
  };

  switch (triggerType) {
    case EventTriggerType.Subscription:
      return {
        user,
        tier: SubscriptionTier.Tier1,
        isGift: randomBool(),
      };

    case EventTriggerType.GiftedSubscription:
      return {
        user,
        tier: SubscriptionTier.Tier1,
        cumulativeTotal: randomNumber(1, 12),
        anonymous: randomBool(),
        total: randomNumber(1, 100),
      };
    case EventTriggerType.Bits:
      return {
        user,
        bits: randomNumber(1, 30_000),
        anonymous: randomBool(),
        message: randomArrayItem(BITS_MESSAGES),
      };
    case EventTriggerType.Raid:
      return {
        user,
        viewers: randomNumber(1, 10_000),
      };
    case EventTriggerType.Redeem:
      return {
        user,
        redemptionId: "",
        rewardId: "0000-0000-0000-0000-0000",
        rewardName: randomArrayItem(REWARD_NAMES),
        cost: randomNumber(1, 10) * 100,
        userInput: randomArrayItem(REWARD_MESSAGES),
      };
    case EventTriggerType.AdBreakBegin:
      return {
        user,
        durationSeconds: 60,
      };
    case EventTriggerType.ShoutoutReceive:
      return {
        user,
        viewerCount: randomNumber(10, 1000),
      };

    default:
      return { user };
  }
}
