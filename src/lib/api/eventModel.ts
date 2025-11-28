import type {
  LogId,
  EventId,
  EventLog,
  LogsQuery,
  VEventData,
  UpdateEvent,
  CreateEvent,
  ExecutionId,
  UpdateOrdering,
  EventExecution,
  VEvent as Event,
  ExecutionsQuery,
} from "$lib/api/types";

import { invoke } from "@tauri-apps/api/core";
import { createQuery } from "@tanstack/svelte-query";

import { queryClient } from "./client";

const EVENTS_KEY = ["events"];

function createEventKey(id: EventId) {
  return ["event", id] as const;
}

function createEventExecutionsKey(id: EventId, query?: ExecutionsQuery) {
  if (query === undefined) {
    return ["event-executions", id] as const;
  }
  return ["event-executions", id, query] as const;
}

function createEventLogsKey(id: EventId, query?: LogsQuery) {
  if (query === undefined) {
    return ["event-logs", id] as const;
  }

  return ["event-logs", id, query] as const;
}

// -----------------------------------------------------

function invalidateEventsList() {
  return queryClient.invalidateQueries({ queryKey: EVENTS_KEY });
}

export function getEvents() {
  return invoke<Event[]>("get_events");
}

export function getEventById(eventId: EventId) {
  return invoke<Event | null>("get_event_by_id", { eventId });
}

export function exportEvents(eventIds: EventId[]) {
  return invoke<void>("export_events", { eventIds });
}

export async function importEvents(events: unknown[]) {
  await invoke("import_events", { events });
  invalidateEventsList();
}

export async function createEvent(create: CreateEvent, invalidateList = true) {
  const event = await invoke<Event>("create_event", { create });

  const eventKey = createEventKey(event.id);
  queryClient.setQueryData(eventKey, event);

  if (invalidateList) invalidateEventsList();

  return event;
}

export async function updateEvent(update: UpdateEvent, invalidateList = true) {
  const event = await invoke<Event>("update_event", update);

  const itemKey = createEventKey(event.id);
  queryClient.setQueryData(itemKey, event);

  if (invalidateList) invalidateEventsList();

  return event;
}

export function testEvent(eventId: EventId, eventData: VEventData) {
  return invoke<Event>("test_event_by_id", { eventId, eventData });
}

export async function deleteEvent(eventId: EventId, invalidateList = true) {
  await invoke<void>("delete_event", { eventId });

  const eventKey = createEventKey(eventId);

  // Cancel any queries for the item and clear the current item data
  queryClient.cancelQueries({ queryKey: eventKey });
  queryClient.setQueryData(eventKey, undefined);

  if (invalidateList) invalidateEventsList();
}

export async function deleteEvents(eventIds: EventId[]) {
  await Promise.all(eventIds.map((eventId) => deleteEvent(eventId, false)));

  invalidateEventsList();
}

export async function updateEventOrder(update: UpdateOrdering[]) {
  await invoke("update_event_orderings", { update });
  queryClient.invalidateQueries({ queryKey: EVENTS_KEY });
}

export function getEventExecutions(eventId: EventId, query: ExecutionsQuery) {
  return invoke<EventExecution[]>("get_event_executions", {
    eventId,
    query,
  });
}

export function invalidateEventExecutions(
  eventId: EventId,
  query: ExecutionsQuery,
) {
  const queryKey = createEventExecutionsKey(eventId, query);
  queryClient.invalidateQueries({ queryKey });
}

export async function deleteEventExecutions(
  eventId: EventId,
  executionIds: ExecutionId[],
) {
  await invoke<void>("delete_event_executions", { executionIds });

  queryClient.setQueriesData<EventExecution[]>(
    { queryKey: createEventExecutionsKey(eventId) },
    (data) => {
      if (data === undefined) return undefined;
      return data.filter((log) => !executionIds.includes(log.id));
    },
  );

  // Invalid the list of items
  queryClient.invalidateQueries({
    queryKey: createEventExecutionsKey(eventId),
  });
}

export function getEventLogs(eventId: EventId, query: LogsQuery) {
  return invoke<EventLog[]>("get_event_logs", { eventId, query });
}

export function invalidateEventLogs(eventId: EventId, query: LogsQuery) {
  const queryKey = createEventLogsKey(eventId, query);
  queryClient.invalidateQueries({ queryKey });
}

export async function deleteEventLogs(eventId: EventId, logIds: LogId[]) {
  await invoke<void>("delete_event_logs", { logIds });

  queryClient.setQueriesData<EventLog[]>(
    { queryKey: createEventLogsKey(eventId) },
    (data) => {
      if (data === undefined) return undefined;
      return data.filter((log) => !logIds.includes(log.id));
    },
  );

  // Invalid the list of items
  queryClient.invalidateQueries({
    queryKey: createEventLogsKey(eventId),
  });
}

// -----------------------------------------------------

export function eventExecutionsQuery(
  params: () => { eventId: EventId; query: ExecutionsQuery },
) {
  return createQuery(() => {
    const { eventId, query } = params();

    return {
      queryKey: createEventExecutionsKey(eventId, query),
      queryFn: () => getEventExecutions(eventId, query),
    };
  });
}

export function createEventsQuery() {
  return createQuery(() => ({
    queryKey: EVENTS_KEY,
    queryFn: getEvents,
  }));
}

export function createEventQuery(idParam: () => EventId) {
  return createQuery(() => {
    const id = idParam();

    return {
      queryKey: createEventKey(id),
      queryFn: () => getEventById(id),
    };
  });
}

export function eventLogsQuery(
  params: () => { eventId: EventId; query: LogsQuery },
) {
  return createQuery(() => {
    const { eventId, query } = params();

    return {
      queryKey: createEventLogsKey(eventId, query),
      queryFn: () => getEventLogs(eventId, query),
    };
  });
}
