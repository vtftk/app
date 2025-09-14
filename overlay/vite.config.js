import path from "path";
import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

export default defineConfig(async () => ({
  plugins: [viteSingleFile()],
  resolve: {
    alias: {
      $shared: path.resolve(__dirname, "../types"),
    },
  },
}));
