<script lang="ts">
  import { toast } from "svelte-sonner";
  import Label from "$lib/components/Label.svelte";
  import { setClipboard } from "$lib/utils/browser";
  import Card from "$lib/components/card/Card.svelte";
  import { toastErrorMessage } from "$lib/utils/error";
  import Button from "$lib/components/input/Button.svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import {
    getAppContext,
    createOverlayURLQuery,
  } from "$lib/api/runtimeAppData";

  const appContext = getAppContext();
  const runtimeAppData = $derived(appContext.runtimeAppData);

  // Query for the overlay URL
  const overlayURLQuery = createOverlayURLQuery();

  /**
   * Handler for clicking the "Copy Link" button to copy the overlay URL
   */
  function onCopyOverlay() {
    const overlayURL: string | undefined = overlayURLQuery.data;

    if (overlayURL === undefined) return;

    const copyPromise = setClipboard(overlayURL);
    toast.promise(copyPromise, {
      loading: "Copying overlay URL...",
      success: "Copied overlay URL",
      error: toastErrorMessage("Failed to copy overlay URL"),
    });
  }
</script>

<Card>
  <div class="status-item">
    <svg
      class="status-item-logo"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
    >
      <path
        fill="#9d9c9c"
        d="M12 24C5.383 24 0 18.617 0 12S5.383 0 12 0s12 5.383 12 12s-5.383 12-12 12m0-22.891C5.995 1.109 1.11 5.995 1.11 12S5.995 22.89 12 22.89S22.89 18.005 22.89 12S18.005 1.109 12 1.109M6.182 5.99c.352-1.698 1.503-3.229 3.05-3.996c-.269.273-.595.483-.844.78c-1.02 1.1-1.48 2.692-1.199 4.156c.355 2.235 2.455 4.06 4.732 4.028c1.765.079 3.485-.937 4.348-2.468c1.848.063 3.645 1.017 4.7 2.548c.54.799.962 1.736.991 2.711c-.342-1.295-1.202-2.446-2.375-3.095a4.9 4.9 0 0 0-3.772-.425c-1.56.448-2.849 1.723-3.293 3.293c-.377 1.25-.216 2.628.377 3.772c-.825 1.429-2.315 2.449-3.932 2.756c-1.244.261-2.551.059-3.709-.464c1.036.302 2.161.355 3.191-.011a4.91 4.91 0 0 0 3.024-2.935c.556-1.49.345-3.261-.591-4.54c-.7-1.007-1.803-1.717-3.002-1.969c-.38-.068-.764-.098-1.148-.134c-.611-1.231-.834-2.66-.528-3.996z"
      />
    </svg>

    <div class="status-text">
      <h2>
        OBS Overlay

        <span class="labels">
          {#if runtimeAppData.active_overlay_count > 0}
            <Label color="green">Connected</Label>
          {:else}
            <Label color="red">Not Connected</Label>
          {/if}
        </span>
      </h2>

      {#if overlayURLQuery.data}
        <div class="actions">
          <Button onclick={onCopyOverlay}>Copy Link</Button>
          <p class="url">{overlayURLQuery.data}</p>
        </div>
      {/if}
    </div>

    <StatusIndicator
      status={runtimeAppData.active_overlay_count > 0 ? "green" : "red"}
    >
      {runtimeAppData.active_overlay_count}
    </StatusIndicator>
  </div>
</Card>

<style>
  .labels {
    display: inline-flex;
    flex-flow: row;
    gap: 0.5rem;
    margin-left: 0.5rem;
  }

  .status-item-logo {
    width: 48px;
    margin-right: 1.5rem;
    flex-shrink: 0;
    flex-grow: 0;
  }

  .status-text {
    flex: auto;
    margin-right: 1rem;
  }

  .status-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .status-text h2 {
    font-size: 1rem;
    color: #fff;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .url {
    padding: 0.5rem;
    background-color: #000;
    border: 1px solid #333;
    border-radius: 0.25rem;
    color: #fff;
    flex: auto;
  }
</style>
