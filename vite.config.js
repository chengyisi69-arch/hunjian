import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// 用 1430 避开常用 1420（被其他项目占用）
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1431 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
