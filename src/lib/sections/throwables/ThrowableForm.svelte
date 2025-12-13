<script lang="ts">
  import { createForm } from "felte";
  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import { uploadFile } from "$lib/api/data";
  import { validator } from "@felte/validator-zod";
  import HTabs from "$lib/components/HTabs.svelte";
  import { reporter } from "@felte/reporter-svelte";
  import { toastErrorMessage } from "$lib/utils/error";
  import BallsIcon from "~icons/solar/balls-bold-duotone";
  import { getAppContext } from "$lib/api/runtimeAppData";
  import Button from "$lib/components/input/Button.svelte";
  import { createItem, updateItem } from "$lib/api/itemModel";
  import BallIcon from "~icons/solar/basketball-bold-duotone";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import FormSlider from "$lib/components/form/FormSlider.svelte";
  import { itemSchema, type ItemSchema } from "$lib/schemas/item";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import ImageUpload from "$lib/components/form/ImageUpload.svelte";
  import { testThrow, testThrowBarrage } from "$lib/api/throwables";
  import FormSection from "$lib/components/form/FormSection.svelte";
  import SoundPicker from "$lib/components/sounds/SoundPicker.svelte";
  import SolarAltArrowLeftBold from "~icons/solar/alt-arrow-left-bold";
  import FormTextInput from "$lib/components/form/FormTextInput.svelte";
  import EnabledSwitch from "$lib/components/input/EnabledSwitch.svelte";
  import FormErrorLabel from "$lib/components/form/FormErrorLabel.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import FormNumberInput from "$lib/components/form/FormNumberInput.svelte";
  import SolarSettingsBoldDuotone from "~icons/solar/settings-bold-duotone";
  import SelectedSounds from "$lib/components/sounds/SelectedSounds.svelte";
  import FormBoundCheckbox from "$lib/components/form/FormBoundCheckbox.svelte";
  import SolarGalleryRoundBoldDuotone from "~icons/solar/gallery-round-bold-duotone";
  import SolarHeadphonesRoundBoldDuotone from "~icons/solar/headphones-round-bold-duotone";
  import SolarMultipleForwardRightBoldDuotone from "~icons/solar/multiple-forward-right-bold-duotone";
  import {
    StorageFolder,
    type ItemConfig,
    type ItemWithImpactSounds,
  } from "$lib/api/types";

  type Props = {
    existing?: ItemWithImpactSounds;
  };
  const { existing }: Props = $props();

  const appContext = getAppContext();
  const runtimeAppData = $derived(appContext.runtimeAppData);

  // Testing is only available when an overlay and vtube studio is connected
  const testingEnabled = $derived(
    runtimeAppData.active_overlay_count > 0 &&
      runtimeAppData.vtube_studio_connected,
  );

  function getDefaultItem(): ItemSchema {
    return {
      name: "",
      config: {
        image: {
          image: undefined!,
          scale: 1,
          weight: 1,
          pixelate: false,
        },
        windup: {
          enabled: false,
          duration: 1000,
        },
      },
      impactSoundIds: [],
      windupSoundIds: [],
    };
  }

  function createFromExisting(config: ItemWithImpactSounds): ItemSchema {
    const { image, windup } = config.config;
    return {
      name: config.name,
      config: {
        image: {
          image: image.src,
          scale: image.scale,
          weight: image.weight,
          pixelate: image.pixelate,
        },
        windup: {
          enabled: windup.enabled,
          duration: windup.duration,
        },
      },
      impactSoundIds: config.impact_sounds_ids,
      windupSoundIds: config.windup_sounds_ids,
    };
  }

  const { form, data, touched, setFields, setInitialValues, reset } =
    createForm<ItemSchema>({
      // Derive initial values
      initialValues: getDefaultItem(),

      // Validation and error reporting
      extend: [validator({ schema: itemSchema }), reporter()],

      async onSubmit(values) {
        const savePromise = save(values);

        toast.promise(
          savePromise,
          existing
            ? {
                loading: "Saving item...",
                success: "Saved item",
                error: toastErrorMessage("Failed to save item"),
              }
            : {
                loading: "Creating item...",
                success: "Created item",
                error: toastErrorMessage("Failed to create item"),
              },
        );

        // Go back to the list when creating rather than editing
        if (!existing) {
          goto(resolve("/throwables"));
        }
      },
    });

  $effect(() => {
    setInitialValues(
      existing ? createFromExisting(existing) : getDefaultItem(),
    );
    reset();
  });

  // Store initial impact sounds list for checking touched state
  const initialImpactSoundIds = $data.impactSoundIds;

  // Touched state for impact sound IDs must be manually updated
  $effect(() => {
    if (initialImpactSoundIds !== $data.impactSoundIds) {
      $touched.impactSoundIds = true;
    }
  });

  function saveImage(image: string | File) {
    if (image instanceof File) {
      // Upload new image
      return uploadFile(StorageFolder.ThrowableImage, image);
    }

    // Using existing uploaded image
    return Promise.resolve(image);
  }

  async function save(values: ItemSchema) {
    const { image, windup } = values.config;
    const imageURL: string = await saveImage(image.image);

    const config: ItemConfig = {
      image: {
        src: imageURL,
        pixelate: image.pixelate,
        scale: image.scale,
        weight: image.weight,
      },
      windup: {
        enabled: windup.enabled,
        duration: windup.duration,
      },
    };

    if (existing) {
      await updateItem({
        itemId: existing.id,
        update: {
          name: values.name,
          config,
          impact_sounds: values.impactSoundIds,
          windup_sounds: values.windupSoundIds,
        },
      });
    } else {
      await createItem({
        name: values.name,
        config,
        impact_sounds: values.impactSoundIds,
        windup_sounds: values.windupSoundIds,
      });
    }
  }

  function onTestThrow() {
    if (existing === undefined) return;

    const throwPromise = testThrow([existing.id], 1);

    toast.promise(throwPromise, {
      loading: "Sending throw...",
      success: "Threw item",
      error: toastErrorMessage("Failed to throw item"),
    });
  }

  function onTestBarrage() {
    if (existing === undefined) return;

    const throwPromise = testThrowBarrage([existing.id], 20, 2, 100);

    toast.promise(throwPromise, {
      loading: "Sending barrage...",
      success: "Threw barrage",
      error: toastErrorMessage("Failed to throw barrage"),
    });
  }
</script>

{#snippet detailsTab()}
  <FormSection>
    <FormTextInput
      id="name"
      name="name"
      label="Name"
      placeholder="Example Throwable..."
      minlength={1}
      required
    />

    <FormSlider
      id="config.image.weight"
      name="config.image.weight"
      label="Weight"
      min={0}
      max={4}
      step={0.1}
      value={$data.config.image.weight}
      description="Weight affects how much force your model is hit with when the item impacts (Default: 1)"
      showTicks
    />
  </FormSection>
{/snippet}

{#snippet imageTab()}
  <FormSection>
    <div class="row-group">
      <div class="column">
        <ImageUpload
          id="config.image.image"
          name="config.image.image"
          value={$data.config.image.image ?? existing?.config.image?.src}
          scale={$data.config.image.scale * 0.5}
          pixelated={$data.config.image.pixelate}
          onChange={(file) => {
            setFields("config.image.image", file!);
          }}
        />
        <FormErrorLabel name="config.image.image" />
      </div>

      <div class="column" style="flex: auto;">
        <FormNumberInput
          id="config.image.scale"
          name="config.image.scale"
          label="Scale"
          min={0.1}
          max={10}
          step={0.1}
        />

        <FormBoundCheckbox
          id="config.image.pixelate"
          name="config.image.pixelate"
          label="Pixelate"
          description="Use this option if your image is pixel art"
        />
      </div>
    </div>
  </FormSection>
{/snippet}

{#snippet soundsTab()}
  <FormSection
    title="Impact Sounds"
    description="Choose selection of sounds that can play when the item impacts"
  >
    <SoundPicker
      description="Choose which sounds should play when this item impacts"
      selected={$data.impactSoundIds}
      onChangeSelected={(sounds) => {
        console.log(sounds);
        setFields(
          "impactSoundIds",
          sounds.map((sound) => sound.id),
          true,
        );
      }}
    />

    {#if $data.impactSoundIds.length > 0}
      <SelectedSounds soundIds={$data.impactSoundIds} />
      <FormErrorLabel name="impactSoundIds" />
    {/if}
  </FormSection>
{/snippet}

{#snippet windupTab()}
  <FormSection
    title="Windup"
    description="Add a windup delay and sound before the item is thrown"
    empty={!$data.config.windup.enabled}
  >
    {#snippet action()}
      <EnabledSwitch
        checked={$data.config.windup.enabled}
        onCheckedChange={(value) =>
          setFields("config.windup.enabled", value, true)}
      />
    {/snippet}

    <FormNumberInput
      id="config.windup.duration"
      name="config.windup.duration"
      label="Duration"
      description="How long the windup should take (ms)"
      min={0}
      step={100}
    />

    <SoundPicker
      description="Choose which sounds can play for the windup (Randomly chosen from the selection)"
      selected={$data.windupSoundIds}
      onChangeSelected={(sounds) => {
        setFields(
          "windupSoundIds",
          sounds.map((sound) => sound.id),
          true,
        );
      }}
    />

    {#if $data.windupSoundIds.length > 0}
      <SelectedSounds soundIds={$data.windupSoundIds} />
      <FormErrorLabel name="windupSoundIds" />
    {/if}
  </FormSection>
{/snippet}

<form use:form>
  <PageLayoutList
    title={existing ? "Edit Throwable" : "Create Throwable"}
    description={existing
      ? `Editing "${existing.name}"`
      : "Create a new item that can be thrown"}
  >
    <!-- Back button -->
    {#snippet beforeTitle()}
      <LinkButton href={resolve("/throwables")}>
        <SolarAltArrowLeftBold />
      </LinkButton>
    {/snippet}

    <!-- End actions -->
    {#snippet actions()}
      {#if existing}
        <!-- Button to test throwable -->
        <PopoverButton disabled={!testingEnabled}>
          {#snippet content()}
            <Button
              type="button"
              onclick={onTestThrow}
              disabled={!testingEnabled}
            >
              <BallIcon /> Test
            </Button>
            <Button
              type="button"
              onclick={onTestBarrage}
              disabled={!testingEnabled}
            >
              <BallsIcon /> Test Barrage
            </Button>
          {/snippet}

          <BallIcon /> Test
        </PopoverButton>
      {/if}
      <Button type="submit">{existing ? "Save" : "Create"}</Button>
    {/snippet}

    <HTabs
      tabs={[
        {
          value: "details",
          icon: SolarSettingsBoldDuotone,
          label: "Details",
          content: detailsTab,
        },
        {
          value: "image",
          icon: SolarGalleryRoundBoldDuotone,
          label: "Image",
          content: imageTab,
        },
        {
          value: "sounds",
          icon: SolarHeadphonesRoundBoldDuotone,
          label: "Sounds",
          content: soundsTab,
        },
        {
          value: "windup",
          icon: SolarMultipleForwardRightBoldDuotone,
          label: "Windup",
          content: windupTab,
        },
      ]}
    />
  </PageLayoutList>
</form>

<style>
  .column {
    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  form {
    height: 100%;
  }

  .row-group {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }
</style>
