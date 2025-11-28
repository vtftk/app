<script lang="ts">
  import { Debounced } from "runed";
  import { toast } from "svelte-sonner";
  import Label from "$lib/components/Label.svelte";
  import Card from "$lib/components/card/Card.svelte";
  import { toastErrorMessage } from "$lib/utils/error";
  import Button from "$lib/components/input/Button.svelte";
  import { getTwitchOAuthURI } from "$lib/api/runtimeAppData";
  import LinkButton from "$lib/components/input/LinkButton.svelte";
  import CardSkeleton from "$lib/components/card/CardSkeleton.svelte";
  import { logout, createIsAuthenticatedQuery } from "$lib/api/twitch";
  import StatusIndicator, {
    type StatusColor,
  } from "$lib/components/StatusIndicator.svelte";

  const isAuthenticated = createIsAuthenticatedQuery();

  // Consistent loading times to prevent flickering
  const isTwitchLoading = new Debounced(() => isAuthenticated.isLoading, 300);

  /**
   * Handle logging out from Twitch
   */
  function onLogoutTwitch() {
    const logoutPromise = logout();

    toast.promise(logoutPromise, {
      loading: "Logging out...",
      success: "Logged out",
      error: toastErrorMessage("Failed to logout"),
    });
  }

  const status: StatusColor = $derived.by(() => {
    if (isAuthenticated.isLoading) {
      return "orange";
    }

    if (isAuthenticated.data) {
      return "green";
    }

    return "red";
  });
</script>

{#if isTwitchLoading.current}
  <CardSkeleton />
{:else}
  <Card>
    <div class="status-item">
      <svg
        class="status-item-logo"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
      >
        <path
          fill="#9d9c9c"
          d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714Z"
        />
      </svg>
      <div class="status-text">
        <h2>
          Twitch

          <span class="labels">
            {#if isAuthenticated.data}
              <Label color="green">Connected</Label>
            {:else}
              <Label color="red">Not Connected</Label>
            {/if}
          </span>
        </h2>

        {#if isAuthenticated.data}
          <div class="actions">
            <Button onclick={onLogoutTwitch}>Logout</Button>
          </div>
        {:else}
          <!-- Not authenticated -->
          <p>
            You are not currently connected to <b>Twitch</b>, please visit the
            link below to allow access. Click "Open in browser" to open the link
            in your default browser.
          </p>

          <div class="actions">
            {#await getTwitchOAuthURI()}
              <div class="skeleton" style="width: 20%; height: 1rem"></div>
            {:then url}
              <LinkButton href={url} target="_blank">
                Open in browser
              </LinkButton>
              <input class="url" type="text" readonly value={url} />
            {/await}
          </div>
        {/if}
      </div>

      <StatusIndicator {status} />
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
