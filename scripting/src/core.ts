/**
 * Generates a UUIDv4
 *
 * @returns The generated UUID
 */
export function uuidv4(): string {
  return Deno.core.ops.op_uuid_v4();
}

/**
 * Provides a promise that will resolve in the
 * provided number of milliseconds
 *
 * @param durationMs The duration in milliseconds
 * @returns Promise that will resolve when the time has elapsed
 */
export async function sleep(durationMs: number) {
  return Deno.core.ops.op_sleep(durationMs);
}
