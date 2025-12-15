const LOCAL_API_BASE = "http://localhost:7070";

const isLocalhost = (hostname: string) =>
  hostname === "localhost" || hostname === "127.0.0.1";

const resolveApiBase = (): string => {
  const envBase = import.meta.env?.VITE_API_BASE;
  if (envBase) return envBase;

  if (typeof window === "undefined") {
    return LOCAL_API_BASE;
  }

  // In dev we expect the backend on localhost:7070 regardless of how the UI is accessed (localhost or LAN IP).
  if (import.meta.env?.DEV) {
    return LOCAL_API_BASE;
  }

  const { origin, hostname } = window.location;
  return isLocalhost(hostname) ? LOCAL_API_BASE : origin;
};

export const API_BASE = resolveApiBase();

const normalizeBase = (value: string) => value.replace(/\/+$/, "");

const toWebSocketBase = (httpBase: string): string => {
  try {
    const url = new URL(httpBase);
    const protocol = url.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${url.host}`;
  } catch {
    return httpBase.replace(/^http/i, "ws");
  }
};

const resolveWebSocketBase = (): string => {
  const envBase = import.meta.env?.VITE_WS_BASE;
  if (envBase) return normalizeBase(envBase);

  return normalizeBase(toWebSocketBase(resolveApiBase()));
};

export const WS_BASE = resolveWebSocketBase();
export const WS_PATH = "/ws/chat";
