import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// Port 1420 : convention Tauri (tauri.conf.json → build.devUrl).
export default defineConfig({
  plugins: [solid()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "esnext",
  },
});
