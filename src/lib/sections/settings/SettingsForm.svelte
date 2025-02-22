<script lang="ts">
  import { z } from "zod";
  import { createForm } from "felte";
  import { toast } from "svelte-sonner";
  import { minMax } from "$lib/utils/validation";
  import { formatBytes } from "$lib/utils/format";
  import { validator } from "@felte/validator-zod";
  import HTabs from "$lib/components/HTabs.svelte";
  import Aside from "$lib/components/Aside.svelte";
  import { reporter } from "@felte/reporter-svelte";
  import { toastErrorMessage } from "$lib/utils/error";
  import Button from "$lib/components/input/Button.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import FormSlider from "$lib/components/form/FormSlider.svelte";
  import FormSection from "$lib/components/form/FormSection.svelte";
  import FormSections from "$lib/components/form/FormSections.svelte";
  import SolarBallsBoldDuotone from "~icons/solar/balls-bold-duotone";
  import FormTextInput from "$lib/components/form/FormTextInput.svelte";
  import SolarSettingsBoldDuotone from "~icons/solar/settings-bold-duotone";
  import FormNumberInput from "$lib/components/form/FormNumberInput.svelte";
  import FormBoundCheckbox from "$lib/components/form/FormBoundCheckbox.svelte";
  import DetectVTubeStudio from "$lib/sections/settings/DetectVTubeStudio.svelte";
  import SolarPeopleNearbyBoldDuotone from "~icons/solar/people-nearby-bold-duotone";
  import SolarHeadphonesRoundBoldDuotone from "~icons/solar/headphones-round-bold-duotone";
  import {
    getAppContext,
    createAppDateMutation,
  } from "$lib/api/runtimeAppData";
  import {
    type AppData,
    EYES_MODE_VALUES,
    THROW_DIRECTION_VALUES,
  } from "$lib/api/types";
  import {
    getLogsEstimateSize,
    getExecutionsEstimateSize,
    getChatHistoryEstimateSize,
  } from "$lib/api/data";

  import EyesModeSelect from "./EyesModeSelect.svelte";
  import ThrowableDirectionSelect from "./ThrowableDirectionSelect.svelte";

  const appContext = getAppContext();
  const appData = $derived(appContext.appData);
  const appDataMutation = createAppDateMutation();

  const schema = z.object({
    // Schema for throwables configuration
    throwables: z.object({
      duration: z.number(),
      spin_speed: minMax,
      throw_angle: minMax,
      direction: z.enum(THROW_DIRECTION_VALUES),
      impact_delay: z.number(),
      item_scale: minMax,
    }),
    // Schema for model related configuration
    model: z.object({
      model_return_time: z.number(),
      eyes_on_hit: z.enum(EYES_MODE_VALUES),
    }),
    // Schema for sound configuration
    sounds: z.object({
      global_volume: z.number(),
    }),
    // Schema for vtube studio configuration
    vtube_studio: z.object({
      host: z.string(),
      port: z.number(),
    }),

    main: z.object({
      minimize_to_tray: z.boolean(),
      clean_logs: z.boolean(),
      clean_logs_days: z.number(),
      clean_executions: z.boolean(),
      clean_executions_days: z.number(),
      clean_chat_history: z.boolean(),
      clean_chat_history_days: z.number(),
      auto_updating: z.boolean(),
      http_port: z.number(),
    }),

    physics: z.object({
      enabled: z.boolean(),
      fps: z.number(),
      gravity_multiplier: z.number(),
      horizontal_multiplier: z.number(),
      vertical_multiplier: z.number(),
    }),
  });

  type Schema = z.infer<typeof schema>;

  function createFromExisting(appData: AppData): Schema {
    const {
      throwables_config,
      model_config,
      sounds_config,
      vtube_studio_config,
      main_config,
      physics_config,
    } = appData;

    return {
      throwables: {
        duration: throwables_config.duration,
        spin_speed: throwables_config.spin_speed,
        throw_angle: throwables_config.throw_angle,
        direction: throwables_config.direction,
        impact_delay: throwables_config.impact_delay,
        item_scale: throwables_config.item_scale,
      },
      model: {
        model_return_time: model_config.model_return_time,
        eyes_on_hit: model_config.eyes_on_hit,
      },
      sounds: {
        global_volume: sounds_config.global_volume,
      },
      vtube_studio: {
        host: vtube_studio_config.host,
        port: vtube_studio_config.port,
      },
      main: {
        minimize_to_tray: main_config.minimize_to_tray,
        clean_logs: main_config.clean_logs,
        clean_logs_days: main_config.clean_logs_days,
        clean_executions: main_config.clean_executions,
        clean_executions_days: main_config.clean_executions_days,
        clean_chat_history: main_config.clean_chat_history,
        clean_chat_history_days: main_config.clean_chat_history_days,
        auto_updating: main_config.auto_updating,
        http_port: main_config.http_port,
      },
      physics: {
        enabled: physics_config.enabled,
        fps: physics_config.fps,
        gravity_multiplier: physics_config.gravity_multiplier,
        horizontal_multiplier: physics_config.horizontal_multiplier,
        vertical_multiplier: physics_config.vertical_multiplier,
      },
    };
  }

  const { form, data, setFields } = $derived(
    createForm<z.infer<typeof schema>>({
      initialValues: createFromExisting(appData),

      // Validation and error reporting
      extend: [validator({ schema }), reporter()],

      async onSubmit(values) {
        const savePromise = save(values);

        toast.promise(savePromise, {
          loading: "Saving settings...",
          success: "Saved settings",
          error: toastErrorMessage("Failed to save settings"),
        });

        await savePromise;
      },
    }),
  );

  async function save(values: Schema) {
    const { throwables, model, sounds, vtube_studio, main, physics } = values;

    await $appDataMutation.mutateAsync({
      ...appData,

      throwables_config: {
        ...appData.throwables_config,
        duration: throwables.duration,
        spin_speed: throwables.spin_speed,
        throw_angle: throwables.throw_angle,
        direction: throwables.direction,
        impact_delay: throwables.impact_delay,
        item_scale: throwables.item_scale,
      },
      model_config: {
        ...appData.model_config,
        model_return_time: model.model_return_time,
        eyes_on_hit: model.eyes_on_hit,
      },
      sounds_config: {
        ...appData.sounds_config,
        global_volume: sounds.global_volume,
      },
      vtube_studio_config: {
        ...appData.vtube_studio_config,
        host: vtube_studio.host,
        port: vtube_studio.port,
      },
      main_config: {
        ...appData.main_config,
        minimize_to_tray: main.minimize_to_tray,
        clean_logs: main.clean_logs,
        clean_logs_days: main.clean_logs_days,
        clean_executions: main.clean_executions,
        clean_executions_days: main.clean_executions_days,
        clean_chat_history: main.clean_chat_history,
        clean_chat_history_days: main.clean_chat_history_days,
        auto_updating: main.auto_updating,
        http_port: main.http_port,
      },
      physics_config: {
        ...appData.physics_config,
        enabled: physics.enabled,
        fps: physics.fps,
        gravity_multiplier: physics.gravity_multiplier,
        horizontal_multiplier: physics.horizontal_multiplier,
        vertical_multiplier: physics.vertical_multiplier,
      },
    });
  }
</script>

{#snippet mainTabContent()}
  <FormSections>
    <FormSection title="App">
      <FormBoundCheckbox
        id="main.auto_updating"
        name="main.auto_updating"
        label="Automatic Updates"
        description="Automatically download and install the latest version when a new version is available"
      />

      <FormBoundCheckbox
        id="main.minimize_to_tray"
        name="main.minimize_to_tray"
        label="Minimize to tray"
        description="Enable minimizing to tray on close instead of closing the app."
      />

      <Aside severity="info">
        "Minimize to tray" allows you to close the app when you're not managing
        to greatly reduce its resource usage. Instead of closing the app the
        close button will minimize the app to your tray menu (The little arrow
        on the bottom right of your screen)
        <br />
        <br />
        Turn off this setting if you want the app to close fully when close is pushed.
      </Aside>
    </FormSection>
    <FormSection
      title="Logging"
      description="VTFTK keeps track of logging messages when running scripts and commands, you can automatically clear them after time has passed in order to save space"
    >
      <p class="helper">
        You can view and delete logs for individual scripts manually using the
        "Logs" tab when editing the script/command
      </p>

      <FormBoundCheckbox
        id="main.clean_logs"
        name="main.clean_logs"
        label="Automatically clean logs"
        description="Disable this to prevent automatic log clearing"
      />

      <FormNumberInput
        id="main.clean_logs_days"
        name="main.clean_logs_days"
        label="Retain days"
        description="Number of days logs will be retained for"
        min={0}
      />
      <p class="size-estimate">
        {#await getLogsEstimateSize()}
          Loading Estimate...
        {:then sizeBytes}
          Estimated Size:
          <span class="size-estimate__size">
            {formatBytes(sizeBytes)}
          </span>
        {/await}
      </p>
    </FormSection>
    <FormSection
      title="Executions"
      description="VTFTK tracks executions of commands and events, this allows it to keep track of cooldown and show you who's triggered a command or event"
    >
      <FormBoundCheckbox
        id="main.clean_executions"
        name="main.clean_executions"
        label="Automatically clean executions"
        description="Disable this to prevent automatic log clearing"
      />

      <FormNumberInput
        id="main.clean_executions_days"
        name="main.clean_executions_days"
        label="Retain days"
        description="Number of days executions will be retained for"
        min={0}
      />

      <p class="size-estimate">
        {#await getExecutionsEstimateSize()}
          Loading Estimate...
        {:then sizeBytes}
          Estimated Size:
          <span class="size-estimate__size">
            {formatBytes(sizeBytes)}
          </span>
        {/await}
      </p>
    </FormSection>
    <FormSection
      title="Chat History"
      description="VTFTK tracks chat history, this allows timers to check if the right number of chat messages have happened before running"
    >
      <FormBoundCheckbox
        id="main.clean_chat_history"
        name="main.clean_chat_history"
        label="Automatically clean chat history"
        description="Disable this to prevent automatic chat history clearing"
      />

      <FormNumberInput
        id="main.clean_chat_history_days"
        name="main.clean_chat_history_days"
        label="Retain days"
        description="Number of days chat history will be retained for"
        min={0}
      />

      <p class="size-estimate">
        {#await getChatHistoryEstimateSize()}
          Loading Estimate...
        {:then sizeBytes}
          Estimated Size:
          <span class="size-estimate__size">
            {formatBytes(sizeBytes)}
          </span>
        {/await}
      </p>
    </FormSection>

    <FormSection
      title="Advanced"
      description="Advanced options for experienced users"
    >
      <div class="row row-ll">
        <FormNumberInput
          id="main.http_port"
          name="main.http_port"
          label="Internal Server Port"
          description="Port the internal server (Overlay, Twitch Authentication, and other internal logic)"
        />

        <Button
          type="button"
          onclick={() => {
            setFields("main.http_port", 58371);
          }}
        >
          Default
        </Button>
      </div>

      <Aside title="IMPORTANT" severity="error">
        You should only change the internal server port if you're having issues
        due to the port being in use. You will need to copy your overlay URL
        again if you change this port.
        <br />
        <br />
        You will need to restart for the port change to take effect
      </Aside>
    </FormSection>
  </FormSections>
{/snippet}

{#snippet throwablesTabContent()}
  <FormSections>
    <FormSection title="Duration and delay">
      <FormNumberInput
        id="throwables.duration"
        name="throwables.duration"
        label="Duration"
        description=" Total time that it should take for a thrown item to hit the target (ms)"
      />

      <FormNumberInput
        id="throwables.impact_delay"
        name="throwables.impact_delay"
        label="Impact Delay"
        description="Delay before the impact is registered (ms)"
      />
    </FormSection>

    <!-- Spin speed -->
    <FormSection title="Spin speed">
      <div class="row spin-speed-row">
        <div class="column column--inputs">
          <FormNumberInput
            id="throwables.spin_speed.min"
            name="throwables.spin_speed.min"
            label="Minimum Spin Speed"
            description="Minimum time to complete a full spin (ms)"
          />

          <FormNumberInput
            id="throwables.spin_speed.max"
            name="throwables.spin_speed.max"
            label="Maximum Spin Speed"
            description="Maximum time to complete a full spin (ms)"
          />
        </div>

        <div class="column">
          <div class="speed-visual">
            <img
              class="speed-visual__item"
              src="/avatar-64x64.png"
              alt=""
              style={`animation-duration: ${$data.throwables.spin_speed.min}ms`}
            />

            <img
              class="speed-visual__item"
              src="/avatar-64x64.png"
              alt=""
              style={`animation-duration: ${$data.throwables.spin_speed.max}ms`}
            />
          </div>
        </div>
      </div>
    </FormSection>

    <FormSection title="Angle and direction">
      <ThrowableDirectionSelect
        id="throwables.direction"
        name="throwables.direction"
        label="Direction"
        description="Which directions the items should come from"
        selected={$data.throwables.direction}
        onChangeSelected={(selected) => {
          setFields("throwables.direction", selected);
        }}
      />

      <!-- Throw angle -->
      <div class="row arrows-row">
        <div class="column arrows-column--inputs">
          <FormSlider
            id="throwables.throw_angle.max"
            name="throwables.throw_angle.max"
            label="Maximum Throw Angle"
            description="Maximum angle an item will be throw at"
            min={-90}
            max={90}
            step={1}
            value={$data.throwables.throw_angle.max}
            oninput={() => {
              if (
                $data.throwables.throw_angle.max <
                $data.throwables.throw_angle.min
              ) {
                $data.throwables.throw_angle.min =
                  $data.throwables.throw_angle.max;
              }
            }}
          />

          <FormSlider
            id="throwables.throw_angle.min"
            name="throwables.throw_angle.min"
            label="Minimum Throw Angle"
            description="Minimum angle an item will be throw at"
            min={-90}
            max={90}
            step={1}
            value={$data.throwables.throw_angle.min}
            oninput={() => {
              if (
                $data.throwables.throw_angle.min >
                $data.throwables.throw_angle.max
              ) {
                $data.throwables.throw_angle.max =
                  $data.throwables.throw_angle.min;
              }
            }}
          />
        </div>
        <div class="column arrows-column--arrows">
          <div class="arrow-container">
            <span class="arrow-text">Items coming from this side</span>

            <div class="arrows">
              <div class="arrow-wrapper">
                <span
                  class="arrow"
                  style={`transform: rotate(${$data.throwables.throw_angle.max}deg);`}
                ></span>
              </div>

              <div class="arrow-wrapper">
                <span
                  class="arrow"
                  style={`transform: rotate(${$data.throwables.throw_angle.min}deg);`}
                ></span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </FormSection>

    <FormSection title="Scale">
      <!-- Item scale -->
      <div class="row">
        <FormNumberInput
          id="throwables.item_scale.min"
          name="throwables.item_scale.min"
          label="Minimum Scale"
          description="Minimum scale applied to an item"
        />

        <FormNumberInput
          id="throwables.item_scale.max"
          name="throwables.item_scale.max"
          label="Maximum Scale"
          description="Maximum scale applied to an item"
        />
      </div>
    </FormSection>

    <FormSection title="Physics">
      <FormBoundCheckbox
        id="physics.enabled"
        name="physics.enabled"
        label="Enabled"
        description="Whether physics are enabled"
      />

      <FormNumberInput
        id="physics.fps"
        name="physics.fps"
        label="FPS"
        description="Frame rate to run the animation at"
        min={0}
        max={120}
        step={10}
      />

      <FormNumberInput
        id="physics.gravity_multiplier"
        name="physics.gravity_multiplier"
        label="Gravity Multiplier"
        description="Multiplier applied to gravity, set to -1 to reverse the direction of gravity"
      />

      <div class="row">
        <FormNumberInput
          id="physics.horizontal_multiplier"
          name="physics.horizontal_multiplier"
          label="Horizontal Multiplier"
          description=""
        />

        <FormNumberInput
          id="physics.vertical_multiplier"
          name="physics.vertical_multiplier"
          label="Vertical Multiplier"
          description=""
        />
      </div>
    </FormSection>
  </FormSections>
{/snippet}

{#snippet soundsTabContent()}
  <FormSections>
    <FormSection>
      <FormSlider
        id="sounds.global_volume"
        name="sounds.global_volume"
        label="Global Volume"
        description="Overall volume of all sounds, including impact sounds"
        min={0}
        max={1}
        step={0.1}
        value={$data.sounds.global_volume}
        showTicks
      />

      <!-- TODO: Sound alerts volume, impact sound volume -->
    </FormSection>
  </FormSections>
{/snippet}

{#snippet vtubeStudioTabContent()}
  <FormSections>
    <FormSection
      title="API Settings"
      description="Details for the VTube Studio API"
    >
      <div class="row row-ll">
        <FormTextInput
          id="vtube_studio.host"
          name="vtube_studio.host"
          label="Host"
          description="Host to use when connecting to VTube Studio"
        />

        <Button
          type="button"
          onclick={() => {
            setFields("vtube_studio.host", "localhost");
          }}
        >
          Default
        </Button>
      </div>

      <FormNumberInput
        id="vtube_studio.port"
        name="vtube_studio.port"
        label="Port"
        description="Port that the VTube Studio API is running on"
      />

      <DetectVTubeStudio
        onChoosePort={(port) => setFields("vtube_studio.port", port)}
      />
    </FormSection>
  </FormSections>
{/snippet}

{#snippet vtubeModelTabContent()}
  <FormSections>
    <FormSection title="Model Settings">
      <FormNumberInput
        id="model.model_return_time"
        name="model.model_return_time"
        label="Return Time"
        description="Time it takes for the model to return to its original position after being hit (ms)"
      />

      <EyesModeSelect
        id="model.eyes_on_hit"
        name="model.eyes_on_hit"
        label="Eyes On Hit"
        description="How the model eyes should react to being hit"
        selected={$data.model.eyes_on_hit}
        onChangeSelected={(selected) => {
          setFields("model.eyes_on_hit", selected);
        }}
      />
    </FormSection>
  </FormSections>
{/snippet}

{#snippet actions()}
  <Button type="submit">Save</Button>
{/snippet}

<form use:form class="container">
  <PageLayoutList
    title="Settings"
    description="Configuration for the entire app"
    {actions}
  >
    <HTabs
      tabs={[
        {
          value: "main",
          icon: SolarSettingsBoldDuotone,
          label: "Main",
          content: mainTabContent,
        },
        {
          value: "throwables",
          icon: SolarBallsBoldDuotone,
          label: "Throwables",
          content: throwablesTabContent,
        },
        {
          value: "sounds",
          icon: SolarHeadphonesRoundBoldDuotone,
          label: "Sounds",
          content: soundsTabContent,
        },
        {
          value: "vtube_studio",
          icon: SolarSettingsBoldDuotone,
          label: "VTube Studio",
          content: vtubeStudioTabContent,
        },
        {
          value: "vtube_model",
          icon: SolarPeopleNearbyBoldDuotone,
          label: "VTuber Model",
          content: vtubeModelTabContent,
        },
      ]}
    />
  </PageLayoutList>
</form>

<style>
  .container {
    position: relative;
    overflow: hidden;
    height: 100%;
  }

  .helper {
    font-size: 0.8rem;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    align-items: center;
    justify-content: center;
  }

  .row-ll {
    grid-template-columns: 3fr 1fr;
  }

  .size-estimate {
    font-size: 0.9rem;
  }

  .size-estimate__size {
    color: #63b4c9;
  }

  .column {
    display: flex;
    gap: 1rem;
    flex-flow: column;
    position: relative;
  }

  .arrows-row {
    justify-content: space-between;
    align-items: stretch;
    width: 100%;
    flex: auto;
    display: flex;
  }

  .spin-speed-row {
    display: flex;
  }

  .arrows-column--inputs,
  .column--inputs {
    flex: auto;
  }

  .arrow-container {
    display: flex;
    align-items: center;
    justify-content: space-around;
    gap: 1rem;
    background-color: #222;
    border: 1px solid #333;
    height: 100%;
    padding: 1rem;
  }

  .arrows {
    display: flex;
    flex-flow: column;
    gap: 2rem;
    border-left: 4px solid #cc00cc;
  }

  .arrow-text {
    max-width: 8rem;
    font-size: 16px;
    color: #999;
    text-align: center;
  }

  .arrow-wrapper {
    position: relative;
    width: 80px;
    height: 60px;
    align-self: center;
    padding-left: 12px;
  }

  .arrow {
    position: absolute;
    top: 25px;
    width: 50px;
    height: 5px;
    background-color: #fff;
    animation: arrow 700ms linear infinite;
    border-radius: 0.5rem;
  }

  .arrow::after,
  .arrow::before {
    content: "";
    position: absolute;
    width: 24px;
    height: 5px;
    right: -4px;
    background-color: #fff;
  }

  .arrow::after {
    top: -7px;
    transform: rotate(45deg);
    border-radius: 0.5rem;
  }

  .arrow::before {
    top: 7px;
    transform: rotate(-45deg);
    border-radius: 0.5rem;
  }

  .speed-visual {
    display: flex;
    flex-flow: column;
    gap: 3rem;
    align-items: center;
    justify-content: space-around;
  }

  .speed-visual__item {
    height: 64px;
    width: 64px;
    animation-name: spinClockwise;
    animation-iteration-count: infinite;
    transform-origin: center;
    animation-timing-function: linear !important;
    animation-fill-mode: both !important;
  }

  @keyframes spinClockwise {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
