import * as monaco from "monaco-editor";

import libraryDefinition from "../../../../scripting/dist/runtime.d.ts?raw";
import builtinLibraryDefinition from "../../../../scripting/types/builtin.d.ts?raw";

const compilerOptions =
  monaco.languages.typescript.javascriptDefaults.getCompilerOptions();

monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
  ...compilerOptions,
  noLib: true,
});

// Built-in JS runtime library definitions (ES6 etc etc)
monaco.languages.typescript.javascriptDefaults.addExtraLib(
  builtinLibraryDefinition,
  "file:///lib.d.ts",
);

// VTFTK scripting library
monaco.languages.typescript.javascriptDefaults.addExtraLib(
  libraryDefinition,
  "file:///global.d.ts",
);

// Disable errors about top level await
monaco.languages.typescript.javascriptDefaults.setDiagnosticsOptions({
  noSemanticValidation: false,
  noSyntaxValidation: false,
});

self.MonacoEnvironment = {
  getWorker: async function (_: string, label: string) {
    switch (label) {
      case "json": {
        const jsonWorker = await import(
          "monaco-editor/esm/vs/language/json/json.worker?worker"
        );
        return new jsonWorker.default();
      }
      case "css":
      case "scss":
      case "less": {
        const cssWorker = await import(
          "monaco-editor/esm/vs/language/css/css.worker?worker"
        );

        return new cssWorker.default();
      }
      // case "html":
      // case "handlebars":
      // case "razor": {
      //   const htmlWorker = await import(
      //     "monaco-editor/esm/vs/language/html/html.worker?worker"
      //   );

      //   return new htmlWorker.default();
      // }
      case "typescript":
      case "javascript": {
        const tsWorker = await import(
          "monaco-editor/esm/vs/language/typescript/ts.worker?worker"
        );
        return new tsWorker.default();
      }
      default: {
        const editorWorker = await import(
          "monaco-editor/esm/vs/editor/editor.worker?worker"
        );
        return new editorWorker.default();
      }
    }
  },
};

export default monaco;
