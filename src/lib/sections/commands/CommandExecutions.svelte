<script lang="ts">
  import { onMount } from "svelte";
  import ExecutionsTable from "$lib/sections/executions/ExecutionsTable.svelte";
  import {
    type CommandId,
    type ExecutionId,
    type ExecutionsQuery,
  } from "$lib/api/types";
  import {
    commandExecutionsQuery,
    deleteCommandExecutions,
    invalidateCommandExecutions,
  } from "$lib/api/commandModel";

  type Props = {
    id: CommandId;
  };

  const { id }: Props = $props();

  const query: ExecutionsQuery = $state({});

  const executionsQuery = commandExecutionsQuery(() => ({
    commandId: id,
    query,
  }));
  const executions = $derived(executionsQuery.data ?? []);

  onMount(() => {
    onRefresh();
  });

  async function onBulkDelete(executionIds: ExecutionId[]) {
    await deleteCommandExecutions(id, executionIds);
  }

  function onRefresh() {
    invalidateCommandExecutions(id, query);
  }
</script>

{#if executionsQuery.isPending}
  <div class="skeleton" style="width: 90%; height: 1.5rem; padding: 1rem"></div>
{/if}

<ExecutionsTable {onRefresh} {onBulkDelete} {executions} />
