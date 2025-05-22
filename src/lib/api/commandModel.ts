import type {
  LogId,
  Command,
  CommandId,
  LogsQuery,
  CommandLog,
  ExecutionId,
  UpdateCommand,
  CreateCommand,
  UpdateOrdering,
  ExecutionsQuery,
  CommandExecution,
  CommandWithAliases,
} from "$lib/api/types";

import { invoke } from "@tauri-apps/api/core";
import { createQuery } from "@tanstack/svelte-query";

import { queryClient } from "./client";

const COMMANDS_KEY = ["commands"];

function createCommandKey(id: CommandId) {
  return ["command", id] as const;
}

function createCommandExecutionsKey(id: CommandId, query?: ExecutionsQuery) {
  if (query === undefined) {
    return ["command-executions", id] as const;
  }
  return ["command-executions", id, query] as const;
}

function createCommandLogsKey(id: CommandId, query?: LogsQuery) {
  if (query === undefined) {
    return ["command-logs", id] as const;
  }
  return ["command-logs", id, query] as const;
}

// -----------------------------------------------------

export function getCommands() {
  return invoke<Command[]>("get_commands");
}

export function getCommandById(commandId: CommandId) {
  return invoke<CommandWithAliases | null>("get_command_by_id", { commandId });
}

export function exportCommands(commandIds: CommandId[]) {
  return invoke<void>("export_commands", { commandIds });
}

export async function importCommands(commands: unknown[]) {
  await invoke("import_commands", { commands });
  invalidateCommandsList();
}

function invalidateCommandsList() {
  return queryClient.invalidateQueries({ queryKey: COMMANDS_KEY });
}

export async function createCommand(
  create: CreateCommand,
  invalidateList = true,
) {
  const command = await invoke<CommandWithAliases>("create_command", {
    create,
  });
  const commandKey = createCommandKey(command.id);
  queryClient.setQueryData(commandKey, command);
  if (invalidateList) invalidateCommandsList();
  return command;
}

export async function updateCommand(update: UpdateCommand) {
  const command = await invoke<CommandWithAliases>("update_command", update);

  // Invalidate the specific item query
  const itemKey = createCommandKey(command.id);
  queryClient.setQueryData(itemKey, command);

  invalidateCommandsList();

  return command;
}

export async function deleteCommand(
  commandId: CommandId,
  invalidateList = true,
) {
  await invoke<void>("delete_command", { commandId });

  const commandKey = createCommandKey(commandId);

  // Cancel any queries for the item and clear the current item data
  queryClient.cancelQueries({ queryKey: commandKey });
  queryClient.setQueryData(commandKey, undefined);

  if (invalidateList) invalidateCommandsList();
}

export async function bulkDeleteCommands(commandIds: CommandId[]) {
  await Promise.all(
    commandIds.map((commandId) => deleteCommand(commandId, false)),
  );

  invalidateCommandsList();
}

export function getCommandLogs(commandId: CommandId, query: LogsQuery) {
  return invoke<CommandLog[]>("get_command_logs", { commandId, query });
}

export function invalidateCommandLogs(commandId: CommandId, query: LogsQuery) {
  const queryKey = createCommandLogsKey(commandId, query);
  queryClient.invalidateQueries({ queryKey });
}

export function deleteCommandLogs(logIds: LogId[]) {
  return invoke<void>("delete_command_logs", { logIds });
}

export async function bulkDeleteCommandLogs(
  commandId: CommandId,
  logIds: LogId[],
) {
  await deleteCommandLogs(logIds);

  queryClient.setQueriesData<CommandLog[]>(
    { queryKey: createCommandLogsKey(commandId) },
    (data) => {
      if (data === undefined) return undefined;
      return data.filter((log) => !logIds.includes(log.id));
    },
  );

  queryClient.invalidateQueries({
    queryKey: createCommandLogsKey(commandId),
  });
}

export async function updateCommandOrder(update: UpdateOrdering[]) {
  await invoke("update_command_orderings", { update });
  invalidateCommandsList();
}

export function getCommandExecutions(
  commandId: CommandId,
  query: ExecutionsQuery,
) {
  return invoke<CommandExecution[]>("get_command_executions", {
    commandId,
    query,
  });
}

export function invalidateCommandExecutions(
  commandId: CommandId,
  query: ExecutionsQuery,
) {
  const queryKey = createCommandExecutionsKey(commandId, query);
  queryClient.invalidateQueries({ queryKey });
}

export async function deleteCommandExecutions(
  commandId: CommandId,
  executionIds: ExecutionId[],
) {
  await invoke<void>("delete_command_executions", { executionIds });

  queryClient.setQueriesData<CommandExecution[]>(
    { queryKey: createCommandExecutionsKey(commandId) },
    (data) => {
      if (data === undefined) return undefined;
      return data.filter((log) => !executionIds.includes(log.id));
    },
  );

  queryClient.invalidateQueries({
    queryKey: createCommandExecutionsKey(commandId),
  });
}

// -----------------------------------------------------

export function createCommandsQuery() {
  return createQuery({
    queryKey: COMMANDS_KEY,
    queryFn: getCommands,
  });
}

export function createCommandQuery(id: CommandId) {
  return createQuery({
    queryKey: createCommandKey(id),
    queryFn: () => getCommandById(id),
  });
}

export function commandLogsQuery(commandId: CommandId, query: LogsQuery) {
  return createQuery({
    queryKey: createCommandLogsKey(commandId, query),
    queryFn: () => getCommandLogs(commandId, query),
  });
}

export function commandExecutionsQuery(
  commandId: CommandId,
  query: ExecutionsQuery,
) {
  return createQuery({
    queryKey: createCommandExecutionsKey(commandId, query),
    queryFn: () => getCommandExecutions(commandId, query),
  });
}
