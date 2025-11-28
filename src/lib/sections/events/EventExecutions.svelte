<script lang="ts">
  import { onMount } from "svelte";
  import ExecutionsTable from "$lib/sections/executions/ExecutionsTable.svelte";
  import {
    type EventId,
    type ExecutionId,
    type ExecutionsQuery,
  } from "$lib/api/types";
  import {
    eventExecutionsQuery,
    deleteEventExecutions,
    invalidateEventExecutions,
  } from "$lib/api/eventModel";

  type Props = {
    id: EventId;
  };

  const { id }: Props = $props();

  const query: ExecutionsQuery = $state({});

  const executionsQuery = eventExecutionsQuery(() => ({ eventId: id, query }));
  const executions = $derived(executionsQuery.data ?? []);

  onMount(() => {
    onRefresh();
  });

  async function onBulkDelete(executionIds: ExecutionId[]) {
    await deleteEventExecutions(id, executionIds);
  }

  function onRefresh() {
    invalidateEventExecutions(id, query);
  }
</script>

{#if executionsQuery.isPending}
  <div class="skeleton" style="width: 90%; height: 1.5rem; padding: 1rem"></div>
{/if}

<ExecutionsTable {onRefresh} {onBulkDelete} {executions} />
