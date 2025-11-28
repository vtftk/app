<script lang="ts">
  import { z } from "zod";
  import { createForm } from "felte";
  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import { validator } from "@felte/validator-zod";
  import HTabs from "$lib/components/HTabs.svelte";
  import { reporter } from "@felte/reporter-svelte";
  import { toastErrorMessage } from "$lib/utils/error";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import Button from "$lib/components/input/Button.svelte";
  import CardButton from "$lib/components/card/CardButton.svelte";
  import PageLayoutList from "$lib/layouts/PageLayoutList.svelte";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import FormSection from "$lib/components/form/FormSection.svelte";
  import CodeEditor from "$lib/components/scripts/CodeEditor.svelte";
  import FormSections from "$lib/components/form/FormSections.svelte";
  import { createCommand, updateCommand } from "$lib/api/commandModel";
  import SolarAltArrowLeftBold from "~icons/solar/alt-arrow-left-bold";
  import FormTextInput from "$lib/components/form/FormTextInput.svelte";
  import TextInputBase from "$lib/components/form/TextInputBase.svelte";
  import EnabledSwitch from "$lib/components/input/EnabledSwitch.svelte";
  import SolarReorderBoldDuotone from "~icons/solar/reorder-bold-duotone";
  import FormNumberInput from "$lib/components/form/FormNumberInput.svelte";
  import SolarSettingsBoldDuotone from "~icons/solar/settings-bold-duotone";
  import TemplateEditor from "$lib/components/scripts/TemplateEditor.svelte";
  import SolarCardSendBoldDuotone from "~icons/solar/card-send-bold-duotone";
  import FormBoundCheckbox from "$lib/components/form/FormBoundCheckbox.svelte";
  import SolarCodeSquareBoldDuotone from "~icons/solar/code-square-bold-duotone";
  import SolarChecklistMinimalisticBoldDuotone from "~icons/solar/checklist-minimalistic-bold-duotone";
  import {
    CommandOutcomeType,
    MinimumRequiredRole,
    type CommandWithAliases,
    MINIMUM_REQUIRED_ROLE_VALUES,
  } from "$lib/api/types";

  import CommandLogs from "./CommandLogs.svelte";
  import CommandExecutions from "./CommandExecutions.svelte";
  import RequiredRoleSelect from "../events/RequiredRoleSelect.svelte";

  const exampleCode = `const { targetUser } = ctx;

// Pick a random message
const message = [
  \`\${targetUser} has been chosen\`,
  \`you picked \${targetUser}\`
].random();

// Return the message to say it
return message;
  `;

  type Props = {
    existing?: CommandWithAliases;
  };

  const { existing }: Props = $props();

  const outcomeSchema = z.discriminatedUnion("type", [
    z.object({
      type: z.literal(CommandOutcomeType.Template),
      message: z
        .string()
        .default("Hey $(user), this is the test command response"),
    }),
    z.object({
      type: z.literal(CommandOutcomeType.Script),
      script: z.string().default(exampleCode),
    }),
  ]);

  const cooldownSchema = z.object({
    enabled: z.boolean(),
    duration: z.number(),
    per_user: z.boolean(),
  });

  const schema = z.object({
    name: z.string().min(1, "You must specify a name"),
    command: z.string().min(1, "You must specify a command"),
    enabled: z.boolean(),
    outcome: outcomeSchema,
    require_role: z.enum(MINIMUM_REQUIRED_ROLE_VALUES),
    cooldown: cooldownSchema,
    aliases: z.array(z.string()),
  });

  type Schema = z.infer<typeof schema>;

  const outcomeState: Record<
    CommandOutcomeType,
    z.infer<typeof outcomeSchema>
  > = {
    [CommandOutcomeType.Template]: getOutcomeDefaults(
      CommandOutcomeType.Template,
    ),
    [CommandOutcomeType.Script]: getOutcomeDefaults(CommandOutcomeType.Script),
  };

  function createFromExisting(config: CommandWithAliases): Schema {
    return {
      name: config.name,
      command: config.command,
      enabled: config.enabled,
      outcome: config.config.outcome,
      require_role: config.config.require_role,
      cooldown: config.config.cooldown,
      aliases: config.aliases.length < 1 ? [""] : config.aliases,
    };
  }

  function getDefaultCommand(): Schema {
    return {
      name: "",
      command: "",
      enabled: true,
      outcome: getOutcomeDefaults(CommandOutcomeType.Template),
      require_role: MinimumRequiredRole.None,
      cooldown: { enabled: true, duration: 1000, per_user: false },
      aliases: [""],
    };
  }

  const {
    form,
    data,
    setFields,
    isDirty,
    setIsDirty,
    setInitialValues,
    reset,
  } = createForm<Schema>({
    // Derive initial values
    initialValues: getDefaultCommand(),

    // Validation and error reporting
    extend: [validator({ schema }), reporter()],

    onSubmit(values) {
      saveWithToast(values);

      if (!existing) {
        goto(resolve("/commands"));
      }
    },
  });

  $effect(() => {
    setInitialValues(
      existing ? createFromExisting(existing) : getDefaultCommand(),
    );
    reset();
  });

  function saveWithToast(values: Schema) {
    const savePromise = save(values);

    toast.promise(
      savePromise,
      existing
        ? {
            loading: "Saving command...",
            success: "Saved command",
            error: toastErrorMessage("Failed to save command"),
          }
        : {
            loading: "Creating command...",
            success: "Created command",
            error: toastErrorMessage("Failed to create command"),
          },
    );

    return savePromise;
  }

  async function save(values: Schema) {
    const command = values.command.toLowerCase().trim();
    const aliases = values.aliases
      .map((alias) => alias.toLowerCase().trim())
      .filter((alias) => alias.length > 0);

    if (existing !== undefined) {
      await updateCommand({
        commandId: existing.id,
        update: {
          enabled: values.enabled,
          name: values.name,
          command,
          config: {
            outcome: values.outcome,
            cooldown: values.cooldown,
            require_role: values.require_role,
          },
          aliases,
        },
      });
    } else {
      await createCommand({
        enabled: values.enabled,
        name: values.name,
        command,
        config: {
          outcome: values.outcome,
          cooldown: values.cooldown,
          require_role: values.require_role,
        },
        aliases,
      });
    }

    setIsDirty(false);
  }

  function getOutcomeDefaults(
    type: CommandOutcomeType,
  ): z.infer<typeof outcomeSchema> {
    switch (type) {
      case CommandOutcomeType.Template:
        return {
          type: CommandOutcomeType.Template,
          message: "Hey $(user), this is the test command response",
        };

      case CommandOutcomeType.Script:
        return {
          type: CommandOutcomeType.Script,
          script: exampleCode,
        };
    }
  }

  function onChangeOutcomeType(type: CommandOutcomeType) {
    // Store current variant state
    outcomeState[$data.outcome.type] = $data.outcome;

    // Swap with new state
    const defaults = outcomeState[type];
    setFields("outcome", defaults, true);
  }

  const commandTypeOption = [
    {
      icon: SolarCodeSquareBoldDuotone,
      color: "red",
      value: CommandOutcomeType.Template,
      label: "Template",
      description:
        "Create a simple text response with some basic templating. Simple commands with static responses",
    },
    {
      icon: SolarCodeSquareBoldDuotone,
      color: "purple",
      value: CommandOutcomeType.Script,
      label: "Script",
      description:
        "Create a command using scripting with JavaScript code. For powerful interactive messages",
    },
  ];

  function handleAddAlias() {
    setFields("aliases", (value) => [...value, ""]);
  }
  function handleRemoveAlias(index: number) {
    setFields("aliases", (value) =>
      value.filter((_, itemIndex) => itemIndex !== index),
    );
  }
</script>

{#snippet settingsTabContent()}
  <FormSection>
    <FormBoundCheckbox
      id="enabled"
      name="enabled"
      label="Enabled"
      description="Whether this command can be used"
    />

    <FormTextInput
      id="name"
      name="name"
      label="Name"
      description="Name for the command"
      placeholder="Test Command"
    />
    <FormTextInput
      id="command"
      name="command"
      label="Command"
      description="Message that will trigger this command"
      placeholder="!test"
    />

    <div>
      <h2 class="aliases-title">Aliases</h2>
      <p class="aliases-description">
        Any of the following alias messages will also trigger the command
      </p>
    </div>

    {#if $data.aliases.length > 0}
      <ul class="aliases">
        {#each $data.aliases as _alias, index (index)}
          <li class="alias">
            <TextInputBase name={`aliases.${index}`} placeholder="!alias" />
            <Button onclick={() => handleRemoveAlias(index)}>
              <DeleteIcon />
            </Button>
          </li>
        {/each}
      </ul>
    {/if}

    <Button onclick={handleAddAlias}>Add Alias</Button>
  </FormSection>
{/snippet}

{#snippet typeTabContent()}
  <div class="event-trigger-grid">
    {#each commandTypeOption as option (option.value)}
      <CardButton
        icon={option.icon}
        color={option.color}
        label={option.label}
        description={option.description}
        selected={$data.outcome.type === option.value}
        onclick={() =>
          $data.outcome.type !== option.value &&
          onChangeOutcomeType(option.value)}
        contentVisible={$data.outcome.type === option.value}
      />
    {/each}
  </div>
{/snippet}

{#snippet codeTabContent()}
  {#if $data.outcome.type === CommandOutcomeType.Script}
    <section class="editor">
      <CodeEditor
        value={$data.outcome.script}
        onChange={(value) => {
          setFields("outcome.script", value, true);
          setIsDirty(true);
        }}
        onUserSave={() => {
          if (existing) saveWithToast($data);
        }}
      />
    </section>
  {:else if $data.outcome.type === CommandOutcomeType.Template}
    <TemplateEditor
      value={$data.outcome.message}
      onChange={(value) => {
        setFields("outcome.message", value, true);
        setIsDirty(true);
      }}
      onUserSave={() => {
        if (existing) saveWithToast($data);
      }}
      templates={[
        {
          key: "user",
          description: "Replaced with the name of the user using the command",
        },
        {
          key: "touser",
          description:
            "Replaced with the name of the user this command is targeting (First provided twitch username)",
        },
      ]}
    />
  {/if}
{/snippet}

{#snippet requirementsTabContent()}
  <FormSections>
    <!-- Cooldown and role requirements -->
    <FormSection
      title="Requirements"
      description="Configure requirements for this command to trigger"
    >
      <RequiredRoleSelect
        id="require_role"
        name="require_role"
        label="Minimum Required Role"
        selected={$data.require_role}
        onChangeSelected={(selected) =>
          setFields("require_role", selected, true)}
        description="Minimum required role the user triggering the event must have in order for the event to trigger"
      />
    </FormSection>

    <FormSection
      title="Cooldown"
      description="Configure cooldown between each use of the command"
      empty={!$data.cooldown.enabled}
    >
      {#snippet action()}
        <EnabledSwitch
          checked={$data.cooldown.enabled}
          onCheckedChange={(value) =>
            setFields("cooldown.enabled", value, true)}
        />
      {/snippet}

      <FormNumberInput
        id="cooldown.duration"
        name="cooldown.duration"
        label="Duration"
        description="How long the cooldown should be between each use of the command (ms)"
        min={0}
        step={100}
      />

      <FormBoundCheckbox
        id="cooldown.per_user"
        name="cooldown.per_user"
        label="Per Person"
        description="Whether the cooldown is on a per person basis or a cooldown for everyone"
      />
    </FormSection>
  </FormSections>
{/snippet}

{#snippet executionsTabContent()}
  {#if existing !== undefined}
    <CommandExecutions id={existing.id} />
  {/if}
{/snippet}

{#snippet logsTabContent()}
  {#if existing !== undefined}
    <CommandLogs id={existing.id} />
  {/if}
{/snippet}

<form use:form>
  <PageLayoutList
    title={existing ? "Edit Command" : "Create Command"}
    description={existing
      ? `Editing "${existing.name}"`
      : "Create an event that will trigger some outcome"}
  >
    <!-- Back button -->
    {#snippet beforeTitle()}
      <LinkButton href={resolve("/commands")}>
        <SolarAltArrowLeftBold />
      </LinkButton>
    {/snippet}

    <!-- End actions -->
    {#snippet actions()}
      {#if existing && $isDirty}
        Unsaved changes...
      {/if}

      <Button type="submit">
        {existing ? "Save" : "Create"}
      </Button>
    {/snippet}

    <HTabs
      tabs={[
        {
          value: "details",
          icon: SolarSettingsBoldDuotone,
          label: "Details",
          content: settingsTabContent,
        },
        {
          value: "type",
          icon: SolarCardSendBoldDuotone,
          label: "Type",
          content: typeTabContent,
        },

        {
          value: "code",
          icon: SolarCodeSquareBoldDuotone,
          label:
            $data.outcome.type === CommandOutcomeType.Template
              ? "Template"
              : "Code",
          content: codeTabContent,
          disablePadding: true,
        },
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
  .editor {
    position: relative;
    overflow: hidden;
    height: 100%;
  }

  form {
    height: 100%;
    display: flex;
    flex-flow: column;
  }

  .event-trigger-grid {
    display: grid;

    grid-template-columns: 1fr;
    gap: 0.5rem;
  }

  .aliases {
    display: flex;
    flex-flow: column;
    gap: 1rem;
    list-style: none;
    width: 100%;
  }

  .alias {
    display: flex;
    gap: 1rem;
    width: 100%;
  }

  .alias :global(.form-input) {
    flex: auto;
  }

  .aliases-title {
    font-weight: normal;
    color: #fff;
    font-size: 1rem;
    margin-bottom: 0.25rem;
  }

  .aliases-description {
    color: #ccc;
    font-size: 0.8rem;
  }
</style>
