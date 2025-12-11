const LOCAL_API_BASE = "http://localhost:7080";

const isLocalhost = (hostname: string) =>
  hostname === "localhost" || hostname === "127.0.0.1";

const resolveApiBase = (): string => {
  const envBase = import.meta.env?.VITE_API_BASE;
  if (envBase) return envBase;

  if (typeof window === "undefined") {
    return LOCAL_API_BASE;
  }

  const { origin, hostname } = window.location;
  return isLocalhost(hostname) ? LOCAL_API_BASE : origin;
};

export const API_BASE = resolveApiBase();
