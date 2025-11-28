<script lang="ts">
  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import { toastErrorMessage } from "$lib/utils/error";
  import SettingsIcon from "~icons/solar/settings-bold";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import Button from "$lib/components/input/Button.svelte";
  import SolarMenuDotsBold from "~icons/solar/menu-dots-bold";
  import BallIcon from "~icons/solar/basketball-bold-duotone";
  import { getEventTestingData } from "$lib/utils/eventTestData";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import SolarGiftBoldDuotone from "~icons/solar/gift-bold-duotone";
  import EnabledSwitch from "$lib/components/input/EnabledSwitch.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import { testEvent, deleteEvent, updateEvent } from "$lib/api/eventModel";
  import SolarKeyboardBoldDuotone from "~icons/solar/keyboard-bold-duotone";
  import SolarMoneyBagBoldDuotone from "~icons/solar/money-bag-bold-duotone";
  import SolarStopwatchBoldDuotone from "~icons/solar/stopwatch-bold-duotone";
  import SolarHandMoneyBoldDuotone from "~icons/solar/hand-money-bold-duotone";
  import SolarHandHeartBoldDuotone from "~icons/solar/hand-heart-bold-duotone";
  import SolarBasketballBoldDuotone from "~icons/solar/basketball-bold-duotone";
  import SolarBoltCircleBoldDuotone from "~icons/solar/bolt-circle-bold-duotone";
  import SolarTextSquareBoldDuotone from "~icons/solar/text-square-bold-duotone";
  import SolarCodeSquareBoldDuotone from "~icons/solar/code-square-bold-duotone";
  import SolarArrowRightBoldDuotone from "~icons/solar/arrow-right-bold-duotone";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import PopoverCloseButton from "$lib/components/popover/PopoverCloseButton.svelte";
  import SolarSkateboardingBoldDuotone from "~icons/solar/skateboarding-bold-duotone";
  import SolarChatSquareCodeBoldDuotone from "~icons/solar/chat-square-code-bold-duotone";
  import SolarEmojiFunnyCircleBoldDuotone from "~icons/solar/emoji-funny-circle-bold-duotone";
  import SolarUsersGroupRoundedBoldDuotone from "~icons/solar/users-group-rounded-bold-duotone";
  import {
    type VEvent,
    EventOutcomeType,
    EventTriggerType,
  } from "$lib/api/types";
  import SolarHeadphonesRoundSoundBoldDuotone from "~icons/solar/headphones-round-sound-bold-duotone";

  type Props = {
    config: VEvent;

    selected: boolean;
    onToggleSelected: VoidFunction;
  };

  const { config, selected, onToggleSelected }: Props = $props();

  async function onDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description: "Are you sure you want to delete this event?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = deleteEvent(config.id);

    toast.promise(deletePromise, {
      loading: "Deleting event...",
      success: "Deleted event",
      error: toastErrorMessage("Failed to delete event"),
    });
  }

  async function onChangeDisabled(value: boolean) {
    const updatePromise = updateEvent({
      eventId: config.id,
      update: {
        enabled: value,
      },
    });

    toast.promise(updatePromise, {
      loading: value ? "Enabling..." : "Disabling...",
      success: value ? "Enabled " + config.name : "Disabled " + config.name,
      error: toastErrorMessage(
        value ? "Failed to enable" : "Failed to disable",
      ),
    });
  }

  function onTest() {
    if (config === undefined) return;

    const eventData = getEventTestingData(config.config.trigger.type);
    const throwPromise = testEvent(config.id, eventData);

    toast.promise(throwPromise, {
      loading: "Running test event...",
      success: "Tested event",
      error: toastErrorMessage("Failed to test event"),
    });
  }
</script>

{#snippet popoverContent()}
  <PopoverCloseButton type="button" onclick={onTest}>
    <BallIcon /> Test
  </PopoverCloseButton>
  <LinkButton href={resolve("/events/[id]", { id: config.id })}>
    <SettingsIcon /> View
  </LinkButton>
  <Button onclick={onDelete}><DeleteIcon /> Delete</Button>
{/snippet}

<div class="event" data-enabled={config.enabled}>
  <div class="event__base">
    <ControlledCheckbox checked={selected} onCheckedChange={onToggleSelected} />

    <div class="event__content">
      <a
        title={config.name}
        href={resolve("/events/[id]", { id: config.id })}
        class="event__name"
      >
        {config.name}
      </a>
    </div>

    <div class="actions">
      <EnabledSwitch
        checked={config.enabled}
        onCheckedChange={onChangeDisabled}
      />

      <PopoverButton
        content={popoverContent}
        contentProps={{ align: "start", side: "left" }}
      >
        <SolarMenuDotsBold />
      </PopoverButton>
    </div>
  </div>

  <div class="event__detail">
    {#if config.config.trigger.type === EventTriggerType.Redeem}
      <div class="detail" data-color="purple">
        <SolarBoltCircleBoldDuotone />
        Redeem
      </div>
    {:else if config.config.trigger.type === EventTriggerType.Command}
      <div class="detail" data-color="red">
        <SolarTextSquareBoldDuotone />
        Command
      </div>
    {:else if config.config.trigger.type === EventTriggerType.Follow}
      <div class="detail" data-color="yellow">
        <SolarUsersGroupRoundedBoldDuotone />
        Follow
      </div>
    {:else if config.config.trigger.type === EventTriggerType.Subscription}
      <div class="detail" data-color="green">
        <SolarUsersGroupRoundedBoldDuotone />
        Subscription
      </div>
    {:else if config.config.trigger.type === EventTriggerType.GiftedSubscription}
      <div class="detail" data-color="blue">
        <SolarGiftBoldDuotone />
        Gifted Subscription
      </div>
    {:else if config.config.trigger.type === EventTriggerType.Bits}
      <div class="detail" data-color="purple">
        <SolarHandMoneyBoldDuotone />
        Gifted Bits
      </div>
    {:else if config.config.trigger.type === EventTriggerType.Raid}
      <div class="detail" data-color="red">
        <SolarSkateboardingBoldDuotone />
        Raid
      </div>
    {:else if config.config.trigger.type === EventTriggerType.Timer}
      <div class="detail" data-color="green">
        <SolarStopwatchBoldDuotone />
        Timer
      </div>
    {:else if config.config.trigger.type === EventTriggerType.AdBreakBegin}
      <div class="detail" data-color="blue">
        <SolarMoneyBagBoldDuotone />
        Ad Break Started
      </div>
    {:else if config.config.trigger.type === EventTriggerType.ShoutoutReceive}
      <div class="detail" data-color="purple">
        <SolarHandHeartBoldDuotone />
        Shoutout Received
      </div>
    {/if}

    {#if config.config.outcome.type === EventOutcomeType.ThrowBits}
      <div class="detail" data-color="green">
        <SolarHandMoneyBoldDuotone />
        Throw Bits
      </div>
    {:else if config.config.outcome.type === EventOutcomeType.ChannelEmotes}
      <div class="detail" data-color="yellow">
        <SolarEmojiFunnyCircleBoldDuotone />
        Channel Emotes
      </div>
    {:else if config.config.outcome.type === EventOutcomeType.Throwable}
      <div class="detail" data-color="purple">
        <SolarBasketballBoldDuotone />
        Throw Item
      </div>
    {:else if config.config.outcome.type === EventOutcomeType.TriggerHotkey}
      <div class="detail" data-color="red">
        <SolarKeyboardBoldDuotone />
        Trigger Hotkey
      </div>
    {:else if config.config.outcome.type === EventOutcomeType.PlaySound}
      <div class="detail" data-color="yellow">
        <SolarHeadphonesRoundSoundBoldDuotone />
        Play Sound
      </div>
    {:else if config.config.outcome.type === EventOutcomeType.SendChatMessage}
      <div class="detail" data-color="green">
        <SolarChatSquareCodeBoldDuotone />
        Send chat message
      </div>
    {:else if config.config.outcome.type === EventOutcomeType.Script}
      <div class="detail" data-color="purple">
        <SolarCodeSquareBoldDuotone />
        Run script
      </div>
    {/if}

    <span class="detail-transition">
      <SolarArrowRightBoldDuotone />
    </span>
  </div>
</div>

<style>
  .event {
    background-color: #1a1a1a;
    border: 1px solid #2f2f2f;
    border-radius: 5px;

    display: flex;
    flex-flow: column;

    padding: 0.5rem;
    overflow: hidden;
    gap: 0.5rem;

    height: 100px;
  }

  .event__base {
    display: flex;
    justify-content: flex-start;
    align-items: center;
    width: 100%;
    height: 60px;
    gap: 1rem;
  }

  .event__detail {
    display: flex;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
    height: 40px;
    position: relative;
  }

  .event__name {
    color: #fff;
    font-weight: bold;

    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;
    text-decoration: none;
  }

  .event__name:hover {
    text-decoration: underline;
  }

  .event__content {
    display: flex;
    flex: auto;
    align-items: center;
    overflow: hidden;
  }

  .actions {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-shrink: 0;
  }

  .detail-transition {
    position: absolute;
    left: 50%;
    top: 50%;
    color: #999;
    font-size: 0.9rem;
    transform: translate(-50%, -50%);
  }

  .detail {
    opacity: 0.75;
    display: flex;
    gap: 0.5rem;
    margin: 0 0.2rem;
    background-color: #222;
    border: 1px solid #111;
    color: #ccc;
    font-size: 0.9rem;
    font-weight: normal;

    border-radius: 0.25rem;
    padding: 0.25rem 0.75rem;

    vertical-align: middle;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .detail[data-color="purple"] {
    border-color: #dd82f0;
    background-color: #3c1b42;
    color: #dd82f0;
  }

  .detail[data-color="red"] {
    border-color: #f08282;
    background-color: #421b1b;
    color: #f08282;
  }

  .detail[data-color="yellow"] {
    border-color: #eef082;
    background-color: #423f1b;
    color: #f0ee82;
  }

  .detail[data-color="green"] {
    border-color: #a1f082;
    background-color: #1b421b;
    color: #a1f082;
  }

  .detail[data-color="blue"] {
    border-color: #82bbf0;
    background-color: #1b2f42;
    color: #82bbf0;
  }
</style>
