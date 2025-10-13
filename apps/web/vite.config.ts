import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  server: {
    port: 3000,
    host: true,
    hmr: {
      clientPort: 3000,
    },
    watch: {
      usePolling: true,
      interval: 500,
    },
    proxy: {
      '/api': {
        target: 'http://localhost:7070',
        changeOrigin: true,
        secure: false,
      },
      '/health': {
        target: 'http://localhost:7070',
        changeOrigin: true,
        secure: false,
      },
      '/ws': {
        target: 'ws://localhost:7070',
        ws: true,
        changeOrigin: true,
      },
    },
  },
  build: {
    target: "esnext"
  }
});
