import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    // The workspace root, so the dev server may serve the shared packages.
    fs: {
      allow: ["../.."]
    }
  },
  ssr: {
    noExternal: ["@pistation/shared-types", "@pistation/client-core", "@pistation/ui"]
  },
  optimizeDeps: {
    include: ["react", "react-dom", "react-dom/client"]
  },
  define: {
    "process.env.IS_PREACT": JSON.stringify("false")
  }
});
