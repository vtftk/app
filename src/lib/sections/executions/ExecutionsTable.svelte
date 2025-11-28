<script lang="ts">
  import { toast } from "svelte-sonner";
  import { toastErrorMessage } from "$lib/utils/error";
  import DeleteIcon from "~icons/solar/trash-bin-2-bold";
  import { formatDate, formatTime } from "$lib/utils/date";
  import Button from "$lib/components/input/Button.svelte";
  import { type ExecutionId, type ExecutionData } from "$lib/api/types";
  import SolarRefreshBoldDuotone from "~icons/solar/refresh-bold-duotone";
  import ControlledCheckbox from "$lib/components/input/ControlledCheckbox.svelte";
  import { confirmDialog } from "$lib/components/dialog/GlobalConfirmDialog.svelte";
  import ExecutionMetadataDialog from "$lib/sections/executions/ExecutionMetadataDialog.svelte";

  type Props = {
    executions: ExecutionData[];

    onRefresh: VoidFunction;
    onBulkDelete: (executionIds: ExecutionId[]) => Promise<void>;
  };

  const {
    executions,
    onRefresh,
    onBulkDelete: _onBulkDelete,
  }: Props = $props();

  let selected: ExecutionId[] = $state([]);

  function onToggleSelected(item: ExecutionId) {
    if (selected.includes(item)) {
      selected = selected.filter((id) => id !== item);
    } else {
      selected = [...selected, item];
    }
  }

  function onToggleAllSelected() {
    if (executions.length > 0 && selected.length === executions.length) {
      selected = [];
    } else {
      selected = executions.map((item) => item.id);
    }
  }

  async function onBulkDelete() {
    const confirm = await confirmDialog({
      title: "Confirm Delete",
      description:
        "Are you sure you want to delete the selected execution entries?",
    });

    if (!confirm) {
      return;
    }

    const deletePromise = _onBulkDelete(selected);

    toast.promise(deletePromise, {
      loading: "Deleting execution entries...",
      success: "Deleted execution entries",
      error: toastErrorMessage("Failed to delete execution entries"),
    });

    // Clear selection since all items are removed
    selected = [];
  }
</script>

<div class="container">
  <div class="selection">
    <div class="selection__count">
      {#if selected.length > 0}
        {selected.length} Selected
      {/if}
    </div>

    <div class="selection__actions">
      {#if selected.length > 0}
        <Button type="button" onclick={onBulkDelete}>
          <DeleteIcon /> Delete
        </Button>
      {/if}

      <Button type="button" onclick={onRefresh}>
        <SolarRefreshBoldDuotone /> Refresh
      </Button>
    </div>
  </div>

  <div class="wrapper">
    <table>
      <thead>
        <tr>
          <th class="column--select">
            <div class="select-actions">
              <ControlledCheckbox
                checked={executions.length > 0 &&
                  selected.length === executions.length}
                onCheckedChange={() => onToggleAllSelected()}
              />
            </div>
          </th>
          <th class="column--msg">User</th>
          <th class="column--msg">Metadata</th>
          <th class="column--date">Timestamp</th>
        </tr>
      </thead>
      <tbody>
        {#each executions as exec (exec.id)}
          <tr>
            <td class="column--select">
              <div class="select-actions">
                <ControlledCheckbox
                  checked={selected.includes(exec.id)}
                  onCheckedChange={() => onToggleSelected(exec.id)}
                />
              </div>
            </td>

            <td class="column--user">
              {#if exec.metadata.user}
                <a
                  class="user-link"
                  target="_blank"
                  href="https://twitch.tv/{exec.metadata.user.name}"
                >
                  {exec.metadata.user.displayName}
                </a>
              {/if}
            </td>
            <td class="column--meta">
              <ExecutionMetadataDialog metadata={exec.metadata} />
            </td>
            <td class="column--date">
              <span class="date-date">
                {formatDate(new Date(exec.created_at))}
              </span>
              <span class="date-time">
                {formatTime(new Date(exec.created_at))}
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .user-link {
    color: #55c0e0;
  }

  .wrapper {
    width: 100%;
    overflow-x: hidden;
    position: relative;
    flex: auto;
  }

  .container {
    display: flex;
    flex-flow: column;
    width: 100%;
    height: 100%;
  }

  table {
    table-layout: fixed;
    width: 100%;
    max-width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    word-wrap: break-word;
    overflow: hidden;
    text-overflow: ellipsis;
    padding: 0.5rem;
    border: 1px solid #333;
    border-top: none;
  }

  td {
    vertical-align: top;
  }

  thead {
    position: sticky;
    top: 0px;
    background-color: #222;
    z-index: 1;
  }

  table th {
    color: #fff;
    vertical-align: center;
  }

  .column--msg {
    text-align: left;
  }

  .column--date {
    width: 7rem;
  }

  .date-date {
    color: #fff;
  }

  .date-time {
    color: #ccc;
    font-size: 0.9rem;
  }

  .column--select {
    width: 3.5rem;
  }

  .select-actions {
    display: flex;
    justify-content: center;
    align-items: center;
    margin: 0.5rem 0;
  }

  .selection {
    display: flex;
    align-items: center;
    gap: 1rem;
    height: 4rem;
    flex-shrink: 0;
    padding: 1rem;
    border: 1px solid #333;
  }

  .selection__count {
    flex: auto;
  }

  .selection__actions {
    display: flex;
    gap: 1rem;
  }
</style>
