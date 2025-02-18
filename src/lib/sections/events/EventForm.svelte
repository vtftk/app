<script lang="ts">
  import { createForm } from "felte";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import { type VEvent } from "$lib/api/types";
  import { validator } from "@felte/validator-zod";
  import HTabs from "$lib/components/HTabs.svelte";
  import { reporter } from "@felte/reporter-svelte";
  import { toastErrorMessage } from "$lib/utils/error";
  import Button from "$lib/components/input/Button.svelte";
  import BallIcon from "~icons/solar/basketball-bold-duotone";
  import { getEventTestingData } from "$lib/utils/eventTestData";
  import CardButton from "$lib/components/card/CardButton.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import FormSection from "$lib/components/form/FormSection.svelte";
  import SolarBookBoldDuotone from "~icons/solar/book-bold-duotone";
  import SolarGiftBoldDuotone from "~icons/solar/gift-bold-duotone";
  import CodeEditor from "$lib/components/scripts/CodeEditor.svelte";
  import FormSections from "$lib/components/form/FormSections.svelte";
  import SolarCard2BoldDuotone from "~icons/solar/card-2-bold-duotone";
  import SolarAltArrowLeftBold from "~icons/solar/alt-arrow-left-bold";
  import FormTextInput from "$lib/components/form/FormTextInput.svelte";
  import EnabledSwitch from "$lib/components/input/EnabledSwitch.svelte";
  import SolarReorderBoldDuotone from "~icons/solar/reorder-bold-duotone";
  import { testEvent, createEvent, updateEvent } from "$lib/api/eventModel";
  import FormNumberInput from "$lib/components/form/FormNumberInput.svelte";
  import SolarKeyboardBoldDuotone from "~icons/solar/keyboard-bold-duotone";
  import TemplateEditor from "$lib/components/scripts/TemplateEditor.svelte";
  import SolarCardSendBoldDuotone from "~icons/solar/card-send-bold-duotone";
  import SolarMoneyBagBoldDuotone from "~icons/solar/money-bag-bold-duotone";
  import SolarStopwatchBoldDuotone from "~icons/solar/stopwatch-bold-duotone";
  import SolarHandMoneyBoldDuotone from "~icons/solar/hand-money-bold-duotone";
  import SolarHandHeartBoldDuotone from "~icons/solar/hand-heart-bold-duotone";
  import FormBoundCheckbox from "$lib/components/form/FormBoundCheckbox.svelte";
  import SolarBasketballBoldDuotone from "~icons/solar/basketball-bold-duotone";
  import ThrowablePicker from "$lib/components/throwable/ThrowablePicker.svelte";
  import SolarCardReciveBoldDuotone from "~icons/solar/card-recive-bold-duotone";
  import SolarBoltCircleBoldDuotone from "~icons/solar/bolt-circle-bold-duotone";
  import SolarTextSquareBoldDuotone from "~icons/solar/text-square-bold-duotone";
  import SolarCodeSquareBoldDuotone from "~icons/solar/code-square-bold-duotone";
  import SolarSkateboardingBoldDuotone from "~icons/solar/skateboarding-bold-duotone";
  import SolarChatSquareCodeBoldDuotone from "~icons/solar/chat-square-code-bold-duotone";
  import SolarEmojiFunnyCircleBoldDuotone from "~icons/solar/emoji-funny-circle-bold-duotone";
  import SolarUsersGroupRoundedBoldDuotone from "~icons/solar/users-group-rounded-bold-duotone";
  import SolarHeadphonesRoundSoundBoldDuotone from "~icons/solar/headphones-round-sound-bold-duotone";
  import SolarChecklistMinimalisticBoldDuotone from "~icons/solar/checklist-minimalistic-bold-duotone";
  import {
    EventOutcomeType,
    EventTriggerType,
    ThrowableDataType,
  } from "$lib/api/types";
  import {
    eventSchema,
    getDefaultEvent,
    type EventSchema,
    getEventTriggerDefaults,
    getEventOutcomeDefaults,
    isEventTriggerWithInput,
    getThrowableDataDefaults,
    type EventTriggerTypeWithInput,
  } from "$lib/schemas/event";

  import EventLogs from "./EventLogs.svelte";
  import SoundSelect from "./SoundSelect.svelte";
  import HotkeySelect from "./HotkeySelect.svelte";
  import EventExecutions from "./EventExecutions.svelte";
  import RequiredRoleSelect from "./RequiredRoleSelect.svelte";
  import TwitchRedeemSelect from "../twitch/TwitchRedeemSelect.svelte";
  import ThrowableDataTypeSelect from "./ThrowableDataTypeSelect.svelte";

  type Props = {
    existing?: VEvent;
  };

  const { existing }: Props = $props();

  const eventTriggerState = getEventTriggerDefaults();
  const eventOutcomeState = getEventOutcomeDefaults();
  const throwableDataState = getThrowableDataDefaults();

  const { form, data, setFields, isDirty, setIsDirty } =
    createForm<EventSchema>({
      // Derive initial values
      initialValues: existing ? existing : getDefaultEvent(),

      // Validation and error reporting
      extend: [validator({ schema: eventSchema }), reporter()],

      async onSubmit(values) {
        await save(values);

        if (!existing) {
          goto("/events");
        }
      },
    });

  async function save(values: EventSchema) {
    let savePromise: Promise<VEvent>;

    if (existing) {
      savePromise = updateEvent({
        eventId: existing.id,
        update: values,
      });
    } else {
      savePromise = createEvent(values);
    }

    toast.promise(
      savePromise,
      existing
        ? {
            loading: "Saving event...",
            success: "Saved event",
            error: toastErrorMessage("Failed to save event"),
          }
        : {
            loading: "Creating event...",
            success: "Created event",
            error: toastErrorMessage("Failed to create event"),
          },
    );

    await savePromise;

    setIsDirty(false);
  }

  type LabelWithDescription = {
    label: string;
    description: string;
  };

  const EVENT_TRIGGER_INPUT_LABEL: Record<
    EventTriggerTypeWithInput,
    LabelWithDescription
  > = {
    [EventTriggerType.Bits]: {
      label: "Use bits amount",
      description: "Use the amount of bits for the amount of thrown items",
    },
    [EventTriggerType.GiftedSubscription]: {
      label: "Use total gifted subs",
      description:
        "Use the amount of gifted subscriptions for the amount of thrown items",
    },
    [EventTriggerType.Subscription]: {
      label: "Use total gifted subs",
      description:
        "Use the amount of months subscribed for the amount of thrown items",
    },
    [EventTriggerType.Raid]: {
      label: "Use raiders count",
      description: "Use the number of raiders for the amount of thrown items",
    },
  };

  function onChangeTriggerType(type: EventTriggerType) {
    // Already the current value
    if ($data.config.trigger.type === type) return;

    // Store current trigger data
    eventTriggerState[$data.config.trigger.type] = $data.config.trigger;

    // Swap with new state
    const defaults = eventTriggerState[type];
    setFields("config.trigger", defaults, true);

    // Update the outcome state
    onChangeTriggerTypeOutcome(type);
  }

  function onChangeTriggerTypeOutcome(type: EventTriggerType) {
    // Change bits throw outcome to just "Throwable" when not using a bits trigger
    if (
      type !== EventTriggerType.Bits &&
      $data.config.outcome.type === EventOutcomeType.ThrowBits
    ) {
      onChangeOutcomeType(EventOutcomeType.Throwable);
      return;
    }

    // Disable "use_input_amount" when trigger becomes a trigger that
    // does not produce an input amount
    if (
      $data.config.outcome.type === EventOutcomeType.Throwable &&
      !isEventTriggerWithInput(type)
    ) {
      const tData = $data.config.outcome.amount;

      if (
        (tData.type === ThrowableDataType.Throw ||
          tData.type === ThrowableDataType.Barrage) &&
        tData.use_input_amount
      ) {
        setFields("config.outcome.amount.use_input_amount", false, true);
      }
    }
  }

  function onChangeOutcomeType(type: EventOutcomeType) {
    // Already the current time
    if ($data.config.outcome.type === type) return;

    // Store current trigger data
    eventOutcomeState[$data.config.outcome.type] = $data.config.outcome;

    // Swap with new state
    const defaults = eventOutcomeState[type];
    setFields("config.outcome", defaults, true);
  }

  function onChangeThrowableDataType(type: ThrowableDataType) {
    if (
      $data.config.outcome.type === EventOutcomeType.Throwable ||
      $data.config.outcome.type === EventOutcomeType.ThrowBits ||
      $data.config.outcome.type === EventOutcomeType.ChannelEmotes
    ) {
      // Store current trigger data
      throwableDataState[$data.config.outcome.amount.type] =
        $data.config.outcome.amount;

      // Swap with new state
      const defaults = throwableDataState[type];

      setFields("config.outcome.amount", defaults, true);
    }
  }

  function onTest() {
    if (existing === undefined) return;

    const eventData = getEventTestingData($data.config.trigger.type);
    const throwPromise = testEvent(existing.id, eventData);

    toast.promise(throwPromise, {
      loading: "Running test event...",
      success: "Tested event",
      error: toastErrorMessage("Failed to test event"),
    });
  }
</script>

{#snippet redeemContent()}
  {#if $data.config.trigger.type === EventTriggerType.Redeem}
    <TwitchRedeemSelect
      name="config.trigger.reward_id"
      label="Reward"
      selected={$data.config.trigger.reward_id}
      onChangeSelected={(selected) =>
        setFields("config.trigger.reward_id", selected, true)}
      description="Choose the twitch redeem that will trigger this event"
    />
  {/if}
{/snippet}

{#snippet commandContent()}
  {#if $data.config.trigger.type === EventTriggerType.Command}
    <FormTextInput
      name="config.trigger.message"
      label="Command phrase"
      description="Triggers when a chat message starting with the provided phrase is received (e.g !test)"
    />
  {/if}
{/snippet}

{#snippet bitsContent()}
  {#if $data.config.trigger.type === EventTriggerType.Bits}
    <FormNumberInput
      name="config.trigger.min_bits"
      label="Minimum Bits"
      description="Minimum number of bits that must be gifted to trigger"
    />
  {/if}
{/snippet}

{#snippet raidContent()}
  {#if $data.config.trigger.type === EventTriggerType.Raid}
    <FormNumberInput
      name="config.trigger.min_raiders"
      label="Minimum Raiders"
      description="Minimum number of people that must be apart of the raid to trigger"
      min={0}
    />
  {/if}
{/snippet}

{#snippet timerContent()}
  {#if $data.config.trigger.type === EventTriggerType.Timer}
    <FormNumberInput
      name="config.trigger.interval"
      label="Interval"
      description="Time in seconds between each trigger of the timer"
      min={1}
    />
    <FormNumberInput
      name="config.trigger.min_chat_messages"
      label="Minimum Chat messages"
      description="Minimum chat messages that must come before the event will trigger (Excludes streamer messages)"
      min={0}
    />
  {/if}
{/snippet}

{#snippet shoutoutReceiveContent()}
  {#if $data.config.trigger.type === EventTriggerType.ShoutoutReceive}
    <FormNumberInput
      name="config.trigger.min_viewers"
      label="Minimum Viewers"
      description="Minimum viewers the channel must have when giving the shoutout"
    />
  {/if}
{/snippet}

{#snippet outcomeThrowableAmount()}
  {#if $data.config.outcome.type === EventOutcomeType.Throwable || $data.config.outcome.type === EventOutcomeType.ThrowBits || $data.config.outcome.type === EventOutcomeType.ChannelEmotes}
    {#if isEventTriggerWithInput($data.config.trigger.type)}
      {@const { label, description } =
        EVENT_TRIGGER_INPUT_LABEL[$data.config.trigger.type]!}
      <!-- Option to use input amount -->
      <FormBoundCheckbox
        name="config.outcome.amount.use_input_amount"
        {label}
        {description}
      />
    {/if}

    {#if isEventTriggerWithInput($data.config.trigger.type) && $data.config.outcome.amount.use_input_amount}
      <!-- Config for picking from input -->
      <div class="throwable-config-grid">
        <FormNumberInput
          name="config.outcome.amount.input_amount_config.multiplier"
          label="Multiplier"
          description="Multiplier applied against the amount"
          min={1}
          step={0.1}
          max={100}
        />
        <FormNumberInput
          name="config.outcome.amount.input_amount_config.range.min"
          label="Minimum Amount"
          description="Minimum amount of items to throw"
          min={1}
          step={1}
          max={1000}
        />
        <FormNumberInput
          name="config.outcome.amount.input_amount_config.range.max"
          label="Maximum Amount"
          description="Maximum amount of items to throw"
          min={1}
          step={1}
          max={1000}
        />
      </div>
    {:else}
      <!-- Single amount picker -->
      <FormNumberInput
        name="config.outcome.amount.amount"
        label="Total number of items to throw"
        description="Total number of items to throw for the whole barrage"
        min={1}
      />
    {/if}
  {/if}
{/snippet}

{#snippet throwBitsOutcomeContent()}
  {#if $data.config.outcome.type === EventOutcomeType.ThrowBits}
    <ThrowableDataTypeSelect
      name="config.outcome.amount.type"
      label="Throwable Type"
      selected={$data.config.outcome.amount.type}
      onChangeSelected={(selected) => {
        onChangeThrowableDataType(selected);
      }}
    />

    {#if $data.config.outcome.amount.type === ThrowableDataType.Throw}
      {@render outcomeThrowableAmount()}

      <p>
        {$data.config.outcome.amount.amount} random item{$data.config.outcome
          .amount.amount > 1
          ? "s"
          : ""} will be chosen from your selection below and thrown
      </p>
    {:else if $data.config.outcome.amount.type === ThrowableDataType.Barrage}
      <div class="throwable-config-grid">
        <FormNumberInput
          name="config.outcome.amount.amount_per_throw"
          label="Amount per barrage"
          description="How many items to throw in each barrage"
          min={1}
        />

        <FormNumberInput
          name="config.outcome.amount.frequency"
          label="Frequency"
          description="Time between each barrage of items (ms)"
          step={100}
          min={0}
          max={1000 * 60 * 60}
        />
      </div>

      {@render outcomeThrowableAmount()}

      <p>
        {$data.config.outcome.amount.amount_per_throw} bit{$data.config.outcome
          .amount.amount > 1
          ? "s"
          : ""} will be chosen and thrown every {$data.config.outcome.amount
          .frequency}ms {$data.config.outcome.amount.use_input_amount
          ? "until a maximum of " +
            $data.config.outcome.amount.input_amount_config.range.max +
            " have been thrown based on the input "
          : "until a total of " + ($data.config.outcome.amount.amount ?? 1)} item{$data
          .config.outcome.amount.amount > 1
          ? "s"
          : ""} have been thrown
      </p>
    {/if}
  {/if}
{/snippet}

{#snippet channelEmotesOutcomeContent()}
  {#if $data.config.outcome.type === EventOutcomeType.ChannelEmotes}
    <ThrowableDataTypeSelect
      name="config.outcome.amount.type"
      label="Throwable Type"
      selected={$data.config.outcome.amount.type}
      onChangeSelected={(selected) => {
        onChangeThrowableDataType(selected);
      }}
    />

    {#if $data.config.outcome.amount.type === ThrowableDataType.Throw}
      {@render outcomeThrowableAmount()}

      <p>
        {$data.config.outcome.amount.amount} random item{$data.config.outcome
          .amount.amount > 1
          ? "s"
          : ""} will be chosen from your selection below and thrown
      </p>
    {:else if $data.config.outcome.amount.type === ThrowableDataType.Barrage}
      <div class="throwable-config-grid">
        <FormNumberInput
          name="config.outcome.amount.amount_per_throw"
          label="Amount per barrage"
          description="How many items to throw in each barrage"
          min={1}
        />

        <FormNumberInput
          name="config.outcome.amount.frequency"
          label="Frequency"
          description="Time between each barrage of items (ms)"
          step={100}
          min={0}
          max={1000 * 60 * 60}
        />
      </div>

      {@render outcomeThrowableAmount()}

      <p>
        {$data.config.outcome.amount.amount_per_throw} emote{$data.config
          .outcome.amount.amount > 1
          ? "s"
          : ""} will be chosen and thrown every {$data.config.outcome.amount
          .frequency}ms {$data.config.outcome.amount.use_input_amount
          ? "until a maximum of " +
            $data.config.outcome.amount.input_amount_config.range.max +
            " have been thrown based on the input "
          : "until a total of " + ($data.config.outcome.amount.amount ?? 1)} item{$data
          .config.outcome.amount.amount > 1
          ? "s"
          : ""} have been thrown
      </p>
    {/if}
  {/if}
{/snippet}

{#snippet throwableOutcomeContent()}
  {#if $data.config.outcome.type === EventOutcomeType.Throwable}
    <ThrowableDataTypeSelect
      name="config.outcome.amount.type"
      label="Throwable Type"
      selected={$data.config.outcome.amount.type}
      onChangeSelected={(selected) => {
        onChangeThrowableDataType(selected);
      }}
    />

    {#if $data.config.outcome.amount.type === ThrowableDataType.Throw}
      {@render outcomeThrowableAmount()}

      <p>
        {$data.config.outcome.amount.amount} random item{$data.config.outcome
          .amount.amount > 1
          ? "s"
          : ""} will be chosen from your selection below and thrown
      </p>
    {:else if $data.config.outcome.amount.type === ThrowableDataType.Barrage}
      <div class="throwable-config-grid">
        <FormNumberInput
          name="config.outcome.amount.amount_per_throw"
          label="Amount per barrage"
          description="How many items to throw in each barrage"
          min={1}
        />

        <FormNumberInput
          name="config.outcome.amount.frequency"
          label="Frequency"
          description="Time between each barrage of items (ms)"
          step={100}
          min={0}
          max={1000 * 60 * 60}
        />
      </div>

      {@render outcomeThrowableAmount()}

      <p>
        {$data.config.outcome.amount.amount_per_throw} random item{$data.config
          .outcome.amount.amount > 1
          ? "s"
          : ""} will be chosen from your selection below and thrown every {$data
          .config.outcome.amount.frequency}ms {$data.config.outcome.amount
          .use_input_amount
          ? "until a maximum of " +
            $data.config.outcome.amount.input_amount_config.range.max +
            " have been thrown based on the input "
          : "until a total of " + ($data.config.outcome.amount.amount ?? 1)} item{$data
          .config.outcome.amount.amount > 1
          ? "s"
          : ""} have been thrown
      </p>
    {/if}

    <ThrowablePicker
      selected={$data.config.outcome.throwable_ids}
      onChangeSelect={(selected) => {
        setFields("config.outcome.throwable_ids", selected, true);
      }}
    />
  {/if}
{/snippet}

{#snippet triggerHotkeyOutcomeContent()}
  {#if $data.config.outcome.type === EventOutcomeType.TriggerHotkey}
    <HotkeySelect
      name="config.outcome.hotkey_id"
      label="Hotkey"
      selected={$data.config.outcome.hotkey_id}
      onChangeSelected={(selected) =>
        setFields("config.outcome.hotkey_id", selected, true)}
      description="Choose which VTube Studio hotkey to trigger"
    />
  {/if}
{/snippet}

{#snippet playSoundOutcomeContent()}
  {#if $data.config.outcome.type === EventOutcomeType.PlaySound}
    <SoundSelect
      name="config.outcome.sound_id"
      label="Sound"
      selected={$data.config.outcome.sound_id}
      onChangeSelected={(selected) =>
        setFields("config.outcome.sound_id", selected, true)}
    />
  {/if}
{/snippet}

{#snippet detailsTabContent()}
  <!-- Base options -->
  <FormSection>
    <FormTextInput name="name" label="Name" placeholder="Example Event" />
    <FormBoundCheckbox
      name="enabled"
      label="Enabled"
      description="Whether this event will be triggered"
    />
  </FormSection>
{/snippet}

{#snippet triggerTabContent()}
  <div class="event-trigger-grid">
    <CardButton
      icon={SolarBoltCircleBoldDuotone}
      color="purple"
      label="Redeem"
      description="Event will be triggered when a specific channel points redeem is redeemed"
      selected={$data.config.trigger.type === EventTriggerType.Redeem}
      onclick={() => onChangeTriggerType(EventTriggerType.Redeem)}
      content={redeemContent}
    />

    <CardButton
      icon={SolarTextSquareBoldDuotone}
      color="red"
      label="Command"
      description="Event will be triggered when a specific phrase appears at the start of a message"
      selected={$data.config.trigger.type === EventTriggerType.Command}
      onclick={() => onChangeTriggerType(EventTriggerType.Command)}
      content={commandContent}
    />

    <CardButton
      icon={SolarUsersGroupRoundedBoldDuotone}
      color="yellow"
      label="Follow"
      description="Event will be triggered when a user follows the twitch channel"
      selected={$data.config.trigger.type === EventTriggerType.Follow}
      onclick={() => onChangeTriggerType(EventTriggerType.Follow)}
    />

    <CardButton
      icon={SolarCard2BoldDuotone}
      color="green"
      label="Subscription"
      description="Event will be triggered when a user purchases a subscription"
      selected={$data.config.trigger.type === EventTriggerType.Subscription}
      onclick={() => onChangeTriggerType(EventTriggerType.Subscription)}
    />

    <CardButton
      icon={SolarGiftBoldDuotone}
      color="blue"
      label="Gifted Subscription"
      description="Event will be triggered when a user gifts any number of subscriptions"
      selected={$data.config.trigger.type ===
        EventTriggerType.GiftedSubscription}
      onclick={() => onChangeTriggerType(EventTriggerType.GiftedSubscription)}
    />

    <CardButton
      icon={SolarHandMoneyBoldDuotone}
      color="purple"
      label="Bits"
      description="Event will trigger when bits are gifted to the channel"
      selected={$data.config.trigger.type === EventTriggerType.Bits}
      onclick={() => onChangeTriggerType(EventTriggerType.Bits)}
      content={bitsContent}
    />

    <CardButton
      icon={SolarSkateboardingBoldDuotone}
      color="red"
      label="Raid"
      description="Event will trigger when the channel is raided by another channel"
      selected={$data.config.trigger.type === EventTriggerType.Raid}
      onclick={() => onChangeTriggerType(EventTriggerType.Raid)}
      content={raidContent}
    />

    <CardButton
      icon={SolarStopwatchBoldDuotone}
      color="green"
      label="Timer"
      description="Event will trigger on a fixed timer"
      selected={$data.config.trigger.type === EventTriggerType.Timer}
      onclick={() => onChangeTriggerType(EventTriggerType.Timer)}
      content={timerContent}
    />

    <CardButton
      icon={SolarMoneyBagBoldDuotone}
      color="blue"
      label="Ad Break Started"
      description="Event will trigger when an Ad break is started"
      selected={$data.config.trigger.type === EventTriggerType.AdBreakBegin}
      onclick={() => onChangeTriggerType(EventTriggerType.AdBreakBegin)}
    />

    <CardButton
      icon={SolarHandHeartBoldDuotone}
      color="purple"
      label="Shoutout Received"
      description="Event will trigger when another channel gives a shoutout"
      selected={$data.config.trigger.type === EventTriggerType.ShoutoutReceive}
      onclick={() => onChangeTriggerType(EventTriggerType.ShoutoutReceive)}
      content={shoutoutReceiveContent}
    />
  </div>
{/snippet}

{#snippet outcomeTabContent()}
  <div class="event-trigger-grid">
    {#if $data.config.trigger.type === EventTriggerType.Bits}
      <CardButton
        icon={SolarHandMoneyBoldDuotone}
        color="green"
        label="Throw Bits"
        description="Only available when using the bits trigger, will throw bits"
        selected={$data.config.outcome.type === EventOutcomeType.ThrowBits}
        onclick={() => onChangeOutcomeType(EventOutcomeType.ThrowBits)}
        content={throwBitsOutcomeContent}
      />
    {/if}

    {#if $data.config.trigger.type === EventTriggerType.Raid}
      <CardButton
        icon={SolarEmojiFunnyCircleBoldDuotone}
        color="yellow"
        label="Channel Emotes"
        description="Only available when using the raid trigger, will throw the raiding channels emotes"
        selected={$data.config.outcome.type === EventOutcomeType.ChannelEmotes}
        onclick={() => onChangeOutcomeType(EventOutcomeType.ChannelEmotes)}
        content={channelEmotesOutcomeContent}
      />
    {/if}

    <CardButton
      icon={SolarBasketballBoldDuotone}
      color="purple"
      label="Throw Item"
      description="Throw a random item from the specified collection"
      selected={$data.config.outcome.type === EventOutcomeType.Throwable}
      onclick={() => onChangeOutcomeType(EventOutcomeType.Throwable)}
      content={throwableOutcomeContent}
    />

    <CardButton
      icon={SolarKeyboardBoldDuotone}
      color="red"
      label="Trigger Hotkey"
      description="Trigger a VTube studio hotkey"
      selected={$data.config.outcome.type === EventOutcomeType.TriggerHotkey}
      onclick={() => onChangeOutcomeType(EventOutcomeType.TriggerHotkey)}
      content={triggerHotkeyOutcomeContent}
    />

    <CardButton
      icon={SolarHeadphonesRoundSoundBoldDuotone}
      color="yellow"
      label="Play Sound"
      description="Play a sound from the available sounds"
      selected={$data.config.outcome.type === EventOutcomeType.PlaySound}
      onclick={() => onChangeOutcomeType(EventOutcomeType.PlaySound)}
      content={playSoundOutcomeContent}
    />

    <CardButton
      icon={SolarChatSquareCodeBoldDuotone}
      color="green"
      label="Send chat message"
      description="Send a message template to chat"
      selected={$data.config.outcome.type === EventOutcomeType.SendChatMessage}
      onclick={() => onChangeOutcomeType(EventOutcomeType.SendChatMessage)}
    />

    <CardButton
      icon={SolarCodeSquareBoldDuotone}
      color="purple"
      label="Run script"
      description="Execute JavaScript code"
      selected={$data.config.outcome.type === EventOutcomeType.Script}
      onclick={() => onChangeOutcomeType(EventOutcomeType.Script)}
    />
  </div>
{/snippet}

{#snippet requirementsTabContent()}
  <FormSections>
    <!-- Role requirements -->
    <FormSection
      title="Requirements"
      description="Configure requirements for this command to trigger"
    >
      <RequiredRoleSelect
        name="config.require_role"
        label="Minimum Required Role"
        selected={$data.config.require_role}
        onChangeSelected={(selected) =>
          setFields("config.require_role", selected, true)}
        description="Minimum required role the user triggering the event must have in order for the event to trigger"
      />
    </FormSection>
    <!-- Cooldown -->
    <FormSection
      title="Cooldown "
      description="Configure cooldown between each trigger of the event"
      empty={!$data.config.cooldown.enabled}
    >
      {#snippet action()}
        <EnabledSwitch
          checked={$data.config.cooldown.enabled}
          onCheckedChange={(value) =>
            setFields("config.cooldown.enabled", value, true)}
        />
      {/snippet}

      <FormNumberInput
        name="config.cooldown.duration"
        label="Duration"
        description="How long the cooldown should be between each trigger of the event (ms)"
        min={0}
        step={100}
      />

      <FormBoundCheckbox
        name="config.cooldown.per_user"
        label="Per Person"
        description="Whether the cooldown is on a per person basis or a cooldown for everyone"
      />
    </FormSection>

    <!-- Delay -->
    <FormSection
      title="Delay"
      description="Delay before the outcome will occur"
    >
      <FormNumberInput
        name="config.outcome_delay"
        label="Outcome Delay"
        description="Delay before this event will be triggered (ms)"
        min={0}
        step={100}
      />
    </FormSection>
  </FormSections>
{/snippet}

{#snippet codeTabContent()}
  {#if $data.config.outcome.type === EventOutcomeType.Script}
    <section class="editor">
      <CodeEditor
        value={$data.config.outcome.script}
        onChange={(value) => {
          setFields("config.outcome.script", value, true);
          setIsDirty(true);
        }}
        onUserSave={() => {
          if (existing) save($data);
        }}
      />
    </section>
  {:else if $data.config.outcome.type === EventOutcomeType.SendChatMessage}
    <TemplateEditor
      value={$data.config.outcome.template}
      onChange={(value) => {
        setFields("config.outcome.template", value, true);
        setIsDirty(true);
      }}
      onUserSave={() => {
        if (existing) save($data);
      }}
      templates={[
        {
          key: "user",
          description:
            'Replaced with the name of the user who triggered the event. Replaced with "Anonymous" when no username is available',
        },
        ...($data.config.trigger.type === EventTriggerType.Redeem
          ? [
              {
                key: "userInput",
                description:
                  "Replaced with the redeem message for redeems that allow user input",
              },
              {
                key: "rewardName",
                description: "Replaced with the name of the redeemable",
              },
              {
                key: "rewardCost",
                description:
                  "Replaced with the channel points cost of the redeem",
              },
            ]
          : []),
        ...($data.config.trigger.type === EventTriggerType.Bits
          ? [
              {
                key: "userInput",
                description: "Replaced with the bits gift message",
              },
              {
                key: "bits",
                description: "Replaced with the number of bits gifted",
              },
            ]
          : []),
        ...($data.config.trigger.type === EventTriggerType.AdBreakBegin
          ? [
              {
                key: "duration",
                description:
                  "Will be replaced with the ad break duration in seconds",
              },
            ]
          : []),
      ]}
    />
  {/if}
{/snippet}

{#snippet executionsTabContent()}
  {#if existing !== undefined}
    <EventExecutions id={existing.id} />
  {/if}
{/snippet}

{#snippet logsTabContent()}
  {#if existing !== undefined}
    <EventLogs id={existing.id} />
  {/if}
{/snippet}

<form use:form>
  <PageLayoutList
    title={existing ? "Edit Event" : "Create Event"}
    description={existing
      ? `Editing "${existing.name}"`
      : "Create an event that will trigger some outcome"}
  >
    <!-- Back button -->
    {#snippet beforeTitle()}
      <LinkButton href="/events">
        <SolarAltArrowLeftBold />
      </LinkButton>
    {/snippet}

    {#snippet actions()}
      {#if existing && $isDirty}
        Unsaved changes...
      {/if}

      {#if existing}
        <Button type="button" onclick={onTest}>
          <BallIcon /> Test
        </Button>
      {/if}

      <Button type="submit">{existing ? "Save" : "Create"}</Button>
    {/snippet}

    <HTabs
      tabs={[
        {
          value: "details",
          icon: SolarBookBoldDuotone,
          label: "Details",
          content: detailsTabContent,
        },
        {
          value: "trigger",
          icon: SolarCardReciveBoldDuotone,
          label: "Trigger",
          content: triggerTabContent,
        },
        {
          value: "outcome",
          icon: SolarCardSendBoldDuotone,
          label: "Outcome",
          content: outcomeTabContent,
        },

        ...($data.config.outcome.type === EventOutcomeType.SendChatMessage ||
        $data.config.outcome.type === EventOutcomeType.Script
          ? [
              {
                value: "code",
                icon: SolarCodeSquareBoldDuotone,
                label:
                  $data.config.outcome.type === EventOutcomeType.SendChatMessage
                    ? "Template"
                    : "Code",
                content: codeTabContent,
                disablePadding: true,
              },
            ]
          : []),

        {
          value: "requirements",
          icon: SolarChecklistMinimalisticBoldDuotone,
          label: "Requirements",
          content: requirementsTabContent,
        },
        ...(existing !== undefined
          ? [
              {
                value: "executions",
                icon: SolarReorderBoldDuotone,
                label: "Executions",
                content: executionsTabContent,
                disablePadding: true,
              },
            ]
          : []),
        ...(existing !== undefined &&
        existing.config.outcome.type === EventOutcomeType.Script
          ? [
              {
                value: "logs",
                icon: SolarReorderBoldDuotone,
                label: "Logs",
                content: logsTabContent,
                disablePadding: true,
              },
            ]
          : []),
      ]}
    />
  </PageLayoutList>
</form>

<style>
  form {
    height: 100%;
  }

  .throwable-config-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 1rem;
  }

  .event-trigger-grid {
    display: grid;

    grid-template-columns: 1fr;
    gap: 0.5rem;
  }

  .editor {
    position: relative;
    overflow: hidden;
    height: 100%;
  }
</style>
