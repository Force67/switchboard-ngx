import { createSignal } from "solid-js";
import { API_BASE } from "./config";

type ConnectionStatus = "online" | "offline" | "connecting" | "limited";

interface ConnectionState {
  status: ConnectionStatus;
  latencyMs: number | null;
  lastSyncISO: string | null;
  lastCheck: number;
}

const HEALTH_CHECK_TIMEOUT_MS = 5_000;
const LIMITED_LATENCY_THRESHOLD_MS = 1_000;
const MONITOR_INTERVAL_MS = 30_000;
const HEALTH_ENDPOINT = "/api/v1/health";

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
    const timeoutId = setTimeout(() => controller.abort(), HEALTH_CHECK_TIMEOUT_MS);

    const response = await fetch(`${API_BASE}${HEALTH_ENDPOINT}`, {
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
        status: latency > LIMITED_LATENCY_THRESHOLD_MS ? "limited" : "online",
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
  intervalId = window.setInterval(checkConnection, MONITOR_INTERVAL_MS);
};

const stopConnectionMonitoring = () => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
  monitoringStarted = false;
};

export { connectionState, checkConnection, startConnectionMonitoring, stopConnectionMonitoring };
