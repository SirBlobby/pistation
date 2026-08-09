import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig, searchForWorkspaceRoot } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],

  ssr: {
    noExternal: ["@pistation/shared-types", "@pistation/client-core", "@pistation/ui"]
  },

  optimizeDeps: {
    include: ["react", "react-dom", "react-dom/client"]
  },

  define: {
    "process.env.IS_PREACT": JSON.stringify("false")
  },

  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd())]
    },
    watch: {
      ignored: ["**/src-tauri/**"]
    }
  }
});
