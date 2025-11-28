<script lang="ts">
  import Dialog from "$lib/components/dialog/Dialog.svelte";
  import MonacoEditor from "$lib/components/scripts/MonacoEditor.svelte";
  import DialogCloseButton from "$lib/components/dialog/DialogCloseButton.svelte";

  type Props = {
    metadata: unknown;
  };

  const { metadata }: Props = $props();

  const metadataValue = $derived(JSON.stringify(metadata, undefined, 2));
</script>

<Dialog
  buttonLabel={{ text: "View Metadata" }}
  contentProps={{ class: "metadata-dialog-content" }}
>
  <!-- Title -->
  {#snippet title()}Metadata{/snippet}

  <!-- Action buttons -->
  {#snippet actions()}
    <DialogCloseButton buttonLabel={{ text: "Close" }} />
  {/snippet}

  <!-- Content -->
  <section class="editor">
    <MonacoEditor
      language="json"
      value={metadataValue}
      readOnly
      onChange={() => {}}
    />
  </section>
</Dialog>

<style>
  .editor {
    position: relative;
    overflow: hidden;
    height: 70vh;
    max-height: 30rem;
  }

  :global(.metadata-dialog-content) {
    width: 60vw;
    max-width: 60rem;
  }
</style>
