import { createSignal } from "solid-js";

const DEFAULT_API_BASE =
  typeof window !== "undefined" ? window.location.origin : "http://localhost:7070";
const API_BASE = import.meta.env.VITE_API_BASE ?? DEFAULT_API_BASE;

type ConnectionStatus = "online" | "offline" | "connecting" | "limited";

interface ConnectionState {
  status: ConnectionStatus;
  latencyMs: number | null;
  lastSyncISO: string | null;
  lastCheck: number;
}

const [connectionState, setConnectionState] = createSignal<ConnectionState>({
  status: "connecting",
  latencyMs: null,
  lastSyncISO: null,
  lastCheck: Date.now(),
});

let intervalId: number | null = null;
let monitoringStarted = false;

const checkConnection = async () => {
  const startTime = Date.now();
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second timeout

    const response = await fetch(`${API_BASE}/health`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      signal: controller.signal,
    });

    clearTimeout(timeoutId);
    const endTime = Date.now();
    const latency = endTime - startTime;

    if (response.ok) {
      const data = await response.json();
      setConnectionState({
        status: latency > 1000 ? "limited" : "online",
        latencyMs: latency,
        lastSyncISO: data.timestamp,
        lastCheck: endTime,
      });
    } else {
      setConnectionState(prev => ({
        ...prev,
        status: "offline",
        lastCheck: endTime,
      }));
    }
  } catch (error) {
    setConnectionState(prev => ({
      ...prev,
      status: "offline",
      lastCheck: Date.now(),
    }));
  }
};

const startConnectionMonitoring = () => {
  if (monitoringStarted) return;
  monitoringStarted = true;

  if (intervalId) return;

  // Initial check
  checkConnection();

  // Check every 30 seconds
  intervalId = window.setInterval(checkConnection, 30000);
};

const stopConnectionMonitoring = () => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
  monitoringStarted = false;
};

export { connectionState, checkConnection, startConnectionMonitoring, stopConnectionMonitoring };
