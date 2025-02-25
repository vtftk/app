<script lang="ts">
  import type * as Monaco from "monaco-editor/esm/vs/editor/editor.api";

  import { onMount, onDestroy } from "svelte";

  type Props = {
    language?: "javascript" | "json" | "commandTemplateFormat";

    readOnly?: boolean;

    value: string;
    onChange: (value: string) => void;

    onUserSave?: VoidFunction;

    options?: Monaco.editor.IStandaloneDiffEditorConstructionOptions;
  };

  const {
    language = "javascript",
    readOnly,
    value,
    onChange,
    onUserSave,
    options,
  }: Props = $props();

  type IMonaco = typeof Monaco;
  type Editor = Monaco.editor.IStandaloneCodeEditor;

  // Access to the monaco lazy loaded library
  let monaco: IMonaco | undefined;

  // Current instance of the editor
  let editor: Editor | undefined;

  // Current container element of the editor
  let editorContainer: HTMLElement | undefined = $state();

  async function loadMonaco() {
    const module = await import("$lib/components/scripts/monaco");
    return module.default;
  }

  function createMonacoEditor(
    monaco: IMonaco,
    language: string,
    readOnly: boolean,
    editorContainer: HTMLElement,
  ) {
    return monaco.editor.create(editorContainer!, {
      theme: "vs-dark",
      automaticLayout: true,
      language,
      tabSize: 2,
      detectIndentation: false,
      fontLigatures: true,
      fixedOverflowWidgets: true,
      fontFamily: "JetBrains Mono",
      readOnly,
      minimap: {
        enabled: false,
      },
      hover: {
        above: false,
      },
      ...options,
    });
  }

  function setupSaveCommand(monaco: IMonaco, editor: Editor) {
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      if (onUserSave) onUserSave();

      // Format on save
      if (editor) editor.getAction("editor.action.formatDocument")?.run();
    });
  }

  onMount(async () => {
    // Load the editor module
    monaco = await loadMonaco();

    editor = createMonacoEditor(
      monaco,
      language,
      readOnly ?? false,
      editorContainer!,
    );

    // Setup save command handling
    setupSaveCommand(monaco, editor);

    const model = monaco.editor.createModel(value, language);
    editor.setModel(model);

    editor.onDidChangeModelContent((event) => {
      const newValue = model.getValue();
      onChange(newValue);
    });
  });

  onDestroy(() => {
    monaco?.editor.getModels().forEach((model) => model.dispose());
    editor?.dispose();
    editor = undefined;
    monaco = undefined;
  });

  // Change editor value when value changes
  $effect(() => {
    editor?.setValue(value);
  });
</script>

<svelte:window
  onresize={() => {
    if (!editor) return;
    // make editor as small as possible
    editor.layout({ width: 0, height: 0 });

    // wait for next frame to ensure last layout finished
    window.requestAnimationFrame(() => {
      if (!editorContainer || !editor) return;
      const rect = editorContainer.getBoundingClientRect();
      editor.layout({ width: rect.width, height: rect.height });
    });
  }}
/>

<div class="container" bind:this={editorContainer}></div>

<style>
  .container {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
