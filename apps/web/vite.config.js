import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

const DEFAULT_DEV_SERVER_PORT = 3000;
const DEFAULT_API_PROXY_TARGET = "http://localhost:7070";
const DEFAULT_WS_PROXY_TARGET = "ws://localhost:7070";
const DEFAULT_WATCH_INTERVAL_MS = 500;

const toNumber = (value, fallback) => {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
};

const devServerPort = toNumber(process.env.VITE_DEV_SERVER_PORT ?? process.env.PORT, DEFAULT_DEV_SERVER_PORT);
const hmrClientPort = toNumber(process.env.VITE_HMR_CLIENT_PORT, devServerPort);
const apiProxyTarget = process.env.VITE_API_BASE ?? DEFAULT_API_PROXY_TARGET;
const wsProxyTarget = process.env.VITE_WS_BASE ?? apiProxyTarget.replace(/^http(s?):/, (_, secure) => secure ? "wss:" : "ws:");
const watchIntervalMs = toNumber(process.env.VITE_WATCH_INTERVAL_MS, DEFAULT_WATCH_INTERVAL_MS);

export default defineConfig({
    plugins: [solid()],
    server: {
        port: devServerPort,
        host: true,
        hmr: {
            clientPort: hmrClientPort,
        },
        watch: {
            usePolling: true,
            interval: watchIntervalMs,
        },
        proxy: {
            '/api': {
                target: apiProxyTarget,
                changeOrigin: true,
                secure: false,
            },
            '/health': {
                target: apiProxyTarget,
                changeOrigin: true,
                secure: false,
            },
            '/ws': {
                target: wsProxyTarget,
                ws: true,
                changeOrigin: true,
            },
        },
    },
    build: {
        target: "esnext"
    }
});
