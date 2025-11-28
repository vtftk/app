<script lang="ts">
  import { Debounced } from "runed";
  import { resolve } from "$app/paths";
  import Label from "$lib/components/Label.svelte";
  import Card from "$lib/components/card/Card.svelte";
  import { createModelDataQuery } from "$lib/api/calibration";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import CardSkeleton from "$lib/components/card/CardSkeleton.svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import { getAppContext, isModelCalibrated } from "$lib/api/runtimeAppData";

  const appContext = getAppContext();
  const runtimeAppData = $derived(appContext.runtimeAppData);

  const modelDataQuery = createModelDataQuery();

  // Model needs to be calibrated if not available here
  const modelCalibrated = $derived(
    isModelCalibrated($modelDataQuery.data ?? [], runtimeAppData.model_id),
  );

  const isCalibrationLoading = new Debounced(
    () => $modelDataQuery.isLoading,
    300,
  );
</script>

{#if isCalibrationLoading.current}
  <CardSkeleton />
{:else}
  <Card>
    <div class="status-item">
      <img
        class="status-item-logo"
        src="/vt-studio.png"
        alt="VTube Studio Logo"
      />
      <div class="status-text">
        <h2>
          VTube Studio

          <span class="labels">
            {#if runtimeAppData.vtube_studio_connected}
              <Label color="green">Connected</Label>
            {:else}
              <Label color="red">Not Connected</Label>
            {/if}
          </span>
        </h2>

        {#if runtimeAppData.vtube_studio_connected}
          {#if runtimeAppData.vtube_studio_auth}
            {#if modelCalibrated}
              <div class="actions">
                <LinkButton href={resolve("/calibration")}>
                  Recalibrate Model
                </LinkButton>
              </div>
            {:else}
              <span class="warning">
                Current model is not calibrated, you must calibrate your model
                in order to throw items.
              </span>

              <div class="actions">
                <LinkButton href={resolve("/calibration")}>
                  Calibrate Model
                </LinkButton>
              </div>
            {/if}
          {:else}
            <p>
              Not Authenticated, please accept the access request prompt within
              VTube Studio
            </p>
          {/if}
        {:else}
          <p>
            Not connected to VTube studio, throwing items will not work. <br />
            Ensure you have the overlay setup in OBS and authorized in VTube Studio
          </p>
        {/if}
      </div>

      <StatusIndicator
        status={runtimeAppData.vtube_studio_connected
          ? modelCalibrated && runtimeAppData.vtube_studio_auth
            ? "green"
            : "orange"
          : "red"}
      />
    </div>
  </Card>
{/if}

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

  .warning {
    display: block;
    color: orange;
    margin-top: 0.5rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
</style>
