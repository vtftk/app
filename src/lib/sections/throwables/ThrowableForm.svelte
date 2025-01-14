<script lang="ts">
  import { z } from "zod";
  import { createForm } from "felte";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import { uploadFile } from "$lib/api/data";
  import reporterDom from "@felte/reporter-dom";
  import { validator } from "@felte/validator-zod";
  import { toastErrorMessage } from "$lib/utils/error";
  import { createSoundsQuery } from "$lib/api/soundModel";
  import BallsIcon from "~icons/solar/balls-bold-duotone";
  import Button from "$lib/components/input/Button.svelte";
  import { createItem, updateItem } from "$lib/api/itemModel";
  import { getRuntimeAppData } from "$lib/api/runtimeAppData";
  import BallIcon from "~icons/solar/basketball-bold-duotone";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import FormSlider from "$lib/components/form/FormSlider.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import ImageUpload from "$lib/components/form/ImageUpload.svelte";
  import { testThrow, testThrowBarrage } from "$lib/api/throwables";
  import FormSection from "$lib/components/form/FormSection.svelte";
  import SoundPicker from "$lib/components/sounds/SoundPicker.svelte";
  import FormSections from "$lib/components/form/FormSections.svelte";
  import SolarAltArrowLeftBold from "~icons/solar/alt-arrow-left-bold";
  import FormTextInput from "$lib/components/form/FormTextInput.svelte";
  import FormErrorLabel from "$lib/components/form/FormErrorLabel.svelte";
  import PopoverButton from "$lib/components/popover/PopoverButton.svelte";
  import FormNumberInput from "$lib/components/form/FormNumberInput.svelte";
  import FormBoundCheckbox from "$lib/components/form/FormBoundCheckbox.svelte";
  import {
    FileType,
    type Sound,
    type ItemWithImpactSounds,
    type ThrowableImageConfig,
  } from "$lib/api/types";

  type Props = {
    existing?: ItemWithImpactSounds;
  };

  const { existing }: Props = $props();

  const runtimeAppData = getRuntimeAppData();
  const soundsQuery = createSoundsQuery();

  const sounds = $derived($soundsQuery.data ?? []);

  // Testing is only available when an overlay and vtube studio is connected
  const testingEnabled = $derived(
    $runtimeAppData.active_overlay_count > 0 &&
      $runtimeAppData.vtube_studio_connected,
  );

  // When working with existing configs we allow the file to be a
  // string to account for already uploaded file URLs
  const imageSchema = z
    .instanceof(File, {
      message: "Image file is required",
      fatal: true,
    })
    .or(z.string());

  const schema = z.object({
    name: z.string().min(1, "You must specify a name"),
    image: imageSchema,
    scale: z.number(),
    weight: z.number(),
    pixelate: z.boolean(),
    impactSoundIds: z.array(z.string()),
  });

  type Schema = z.infer<typeof schema>;

  // Defaults when creating a new throwable
  const createDefaults: Partial<Schema> = {
    name: "",
    image: undefined,
    scale: 1,
    weight: 1,
    pixelate: false,
    impactSoundIds: [],
  };

  function createFromExisting(config: ItemWithImpactSounds): Partial<Schema> {
    return {
      name: config.name,
      image: config.image.src,
      scale: config.image.scale,
      weight: config.image.weight,
      pixelate: config.image.pixelate,
      impactSoundIds: config.impact_sounds.map((sound) => sound.id),
    };
  }

  const { form, data, touched, setFields } = createForm<Schema>({
    // Derive initial values
    initialValues: existing ? createFromExisting(existing) : createDefaults,

    // Validation and error reporting
    extend: [validator({ schema }), reporterDom()],

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
        goto("/throwables");
      }
    },
  });

  // Store initial impact sounds list for checking touched state
  const initialImpactSoundIds = $data.impactSoundIds;

  // Touched state for impact sound IDs must be manually updated
  $effect(() => {
    if (initialImpactSoundIds !== $data.impactSoundIds) {
      $touched.impactSoundIds = true;
    }
  });

  const selectedOptions = $derived(
    filterOptionsSelected(sounds, $data.impactSoundIds),
  );

  function filterOptionsSelected(options: Sound[], selected: string[]) {
    return options.filter((option) => selected.includes(option.id));
  }

  function saveImage(image: string | File) {
    if (image instanceof File) {
      // Upload new image
      return uploadFile(FileType.ThrowableImage, image);
    }

    // Using existing uploaded image
    return Promise.resolve(image);
  }

  async function save(values: Schema) {
    const imageURL: string = await saveImage(values.image);
    const imageConfig: ThrowableImageConfig = {
      src: imageURL,
      pixelate: values.pixelate,
      scale: values.scale,
      weight: values.weight,
    };

    if (existing) {
      await updateItem({
        itemId: existing.id,
        update: {
          name: values.name,
          image: imageConfig,
          impact_sounds: values.impactSoundIds,
        },
      });
    } else {
      await createItem({
        name: values.name,
        image: imageConfig,
        impact_sounds: values.impactSoundIds,
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

<form use:form>
  <PageLayoutList
    title={existing ? "Edit Throwable" : "Create Throwable"}
    description={existing
      ? `Editing "${existing.name}"`
      : "Create a new item that can be thrown"}
  >
    <!-- Back button -->
    {#snippet beforeTitle()}
      <LinkButton href="/throwables">
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

    <FormSections>
      <FormSection
        title="Details"
        description="Choose selection of sounds that can play when the item impacts"
      >
        <FormTextInput id="name" name="name" label="Name" />
      </FormSection>

      <FormSection
        title="Image"
        description="The image to use and its configuration"
      >
        <div class="row-group">
          <ImageUpload
            id="image"
            name="image"
            value={$data.image ?? existing?.image?.src}
            scale={$data.scale * 0.5}
            pixelated={$data.pixelate}
          />

          <div class="column">
            <FormNumberInput
              id="scale"
              name="scale"
              label="Scale"
              min={0.1}
              max={10}
              step={0.1}
            />

            <FormSlider
              id="weight"
              name="weight"
              label="Weight"
              min={0}
              max={4}
              step={0.1}
              value={$data.weight}
              description="Weight affects how much force your model is hit with when the item impacts (Default: 1)"
              showTicks
            />

            <FormBoundCheckbox
              id="pixelate"
              name="pixelate"
              label="Pixelate"
              description="Use this option if your image is pixel art"
            />
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Impact Sounds"
        description="Choose selection of sounds that can play when the item impacts"
      >
        <SoundPicker
          description="Choose which sounds should play when this item impacts"
          selected={$data.impactSoundIds}
          onChangeSelected={(soundIds) => {
            setFields(
              "impactSoundIds",
              soundIds.map((sound) => sound.id),
              true,
            );
          }}
        />

        <div class="sounds">
          <p class="sounds__title">Selected Sounds</p>

          <div class="sounds__grid">
            {#each selectedOptions as option}
              <li class="sound-item">
                <p class="sound-item__name">{option.name}</p>
              </li>
            {/each}
          </div>
        </div>

        <FormErrorLabel name="impactSoundIds" />
      </FormSection>
    </FormSections>
  </PageLayoutList>
</form>

<style>
  .sounds {
    display: flex;
    gap: 1rem;
    flex-flow: column;
  }

  .sounds__title {
    color: #fff;
    font-weight: bold;
  }

  .sounds__grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    overflow: hidden;
  }

  .column {
    display: grid;
    grid-template-columns: 1fr;
    flex: auto;
    gap: 1rem;
  }

  .sound-item {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    width: 100%;
    overflow: hidden;
  }

  .sound-item__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  form {
    height: 100%;
  }

  .row-group {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
</style>
