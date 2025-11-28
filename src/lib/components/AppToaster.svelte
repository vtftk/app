<script lang="ts">
  import { Toaster } from "svelte-sonner";
</script>

<Toaster
  theme="dark"
  richColors
  closeButton
  position="bottom-center"
  visibleToasts={3}
  duration={1000}
/>
