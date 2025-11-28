<script lang="ts">
  import { ValidationMessage } from "@felte/reporter-svelte";

  type Props = {
    name: string;
  };

  const { name }: Props = $props();
</script>

<ValidationMessage for={name} let:messages>
  {#if messages && messages.length > 0}
    <ul class="error-label" id="{name}-validation" aria-live="polite">
      {#each messages as message, index (index)}
        <li>{message}</li>
      {/each}
    </ul>
  {/if}
</ValidationMessage>

<style>
  .error-label {
    color: #c53939;
  }

  .error-label {
    list-style: circle;
    list-style-position: inside;
  }
</style>
