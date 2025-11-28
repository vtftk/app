<script lang="ts">
  type EditorTemplate = {
    key: string;
    description: string;
  };

  type Props = {
    value: string;
    onChange: (value: string) => void;
    onUserSave?: VoidFunction;
    templates: EditorTemplate[];
  };

  const { value, onChange, onUserSave, templates }: Props = $props();

  let editor: HTMLDivElement | undefined = $state(undefined);

  function saveSelection(editor: HTMLDivElement) {
    const selection = window.getSelection();
    if (selection === null || selection.rangeCount === 0) return null;

    const range = selection.getRangeAt(0);
    const preSelectionRange = range.cloneRange();
    preSelectionRange.selectNodeContents(editor);
    preSelectionRange.setEnd(range.startContainer, range.startOffset);

    const start = preSelectionRange.toString().length;
    const end = start + range.toString().length;

    return { start, end };
  }

  function restoreSelection(
    editor: HTMLDivElement,
    start: number,
    end: number,
  ) {
    const selection = window.getSelection();
    if (selection === null) return;

    const range = document.createRange();

    let charIndex: number = 0;
    let startNode: Node | null = null;
    let startOffset: number = 0;
    let endNode: Node | null = null;
    let endOffset: number = 0;

    function traverse(node: Node) {
      if (node.nodeType === Node.TEXT_NODE) {
        const nextCharIndex = charIndex + (node.textContent?.length ?? 0);
        if (!startNode && start >= charIndex && start <= nextCharIndex) {
          startNode = node;
          startOffset = start - charIndex;
        }
        if (!endNode && end >= charIndex && end <= nextCharIndex) {
          endNode = node;
          endOffset = end - charIndex;
        }
        charIndex = nextCharIndex;
      } else {
        for (let child of node.childNodes) {
          traverse(child);
        }
      }
    }

    traverse(editor);

    if (startNode && endNode) {
      range.setStart(startNode, startOffset);
      range.setEnd(endNode, endOffset);
      selection.removeAllRanges();
      selection.addRange(range);
    }
  }

  function highlightVariables(value: string) {
    if (!editor) return;

    // Save the current selection
    const selection = saveSelection(editor);

    // Replace $(variable) with highlighted spans
    // eslint-disable-next-line svelte/no-dom-manipulating
    editor.innerHTML = escapeHTML(value).replace(
      /\$\(([^)]+)\)/g,
      '<span class="variable">$($1)</span>',
    );

    // Restore the selection
    if (selection) {
      restoreSelection(editor, selection.start, selection.end);
    }

    // Restore focus to the editor
    editor.focus();
  }

  function escapeHTML(value: string) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(
      "<!doctype html><body>" + value,
      "text/html",
    );
    return doc.body.textContent || doc.body.innerText || "";
  }

  function handleKeyDown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === "s") {
      event.preventDefault();
      if (onUserSave) onUserSave();
    }
  }

  $effect(() => {
    highlightVariables(value);
  });
</script>

<div class="template-split">
  <section class="editor">
    <div
      bind:this={editor}
      class="template-editor"
      contenteditable="true"
      oninput={(event) => {
        if (!editor) return;
        if (event.target === null) return;
        onChange((event.target as HTMLDivElement).innerText);
      }}
      onkeydown={handleKeyDown}
      role="textbox"
      aria-roledescription="textbox"
      tabindex="0"
    ></div>
  </section>

  <div class="hints">
    <h3>Templating</h3>

    <p>The following templates will be replaced if they are found</p>

    <ul class="templates">
      {#each templates as template (template.key)}
        <li class="template">
          <span>$({template.key})</span> - {template.description}
        </li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .template-editor {
    width: 100%;
    height: 100%;

    padding: 0.5rem;
    font-size: 16px;
    line-height: 1.5;
    white-space: pre-wrap;
    outline: none;
    overflow-wrap: break-word;
    background-color: #1e1e1e;
  }

  .template-editor:global(> .variable) {
    color: #e4b654;
  }

  .editor {
    position: relative;
    overflow: hidden;
    height: 100%;
  }

  .template-split {
    display: flex;
    flex-direction: row;
    height: 100%;
  }

  .template-split .editor {
    flex: auto;
  }

  .hints {
    max-width: 14rem;
    padding: 1rem;
    height: 100%;
    overflow: auto;
  }

  .templates {
    list-style: none;
    display: flex;
    flex-flow: column;
    gap: 1rem;
    margin-top: 1rem;
  }

  .template {
    padding: 0.5rem;
    background-color: #1f1f1f;
  }

  .template > span {
    color: #e4b654;
  }
</style>
