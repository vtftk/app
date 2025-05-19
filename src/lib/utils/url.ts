import { type ServerProviderContext } from "$lib/components/ServerProvider.svelte";

export default function getBackendURL(
  context: ServerProviderContext,
  url: string,
) {
  return url.replace("backend://", context.serverURL);
}
