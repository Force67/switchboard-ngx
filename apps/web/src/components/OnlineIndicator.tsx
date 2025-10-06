// components/OnlineIndicator.tsx
import { createSignal, Show, onMount, onCleanup } from "solid-js";
import { connectionState, startConnectionMonitoring, stopConnectionMonitoring } from "../connectionStore";

type Status = "online" | "offline" | "connecting" | "limited";

export default function OnlineIndicator() {
  const [open, setOpen] = createSignal(false);

  onMount(() => {
    startConnectionMonitoring();
  });

  onCleanup(() => {
    stopConnectionMonitoring();
  });

  const label = () => {
    const state = connectionState();
    switch (state.status) {
      case "online": return "Online";
      case "offline": return "Offline";
      case "connecting": return "Reconnecting…";
      case "limited": return "Limited";
    }
  };

  return (
    <div style="position:relative">
      <button
        class={`status ${connectionState().status}`}
        aria-pressed={open()}
        aria-live="polite"
        title={label()}
        onClick={() => setOpen(!open())}
      >
        <span class="dot" aria-hidden="true" />
        <span class="label">{label()}</span>
        <Show when={connectionState().latencyMs != null && connectionState().status === "online"}>
          <span class="latency">{connectionState().latencyMs} ms</span>
        </Show>
      </button>

      <Show when={open()}>
        <div class="status-pop" role="dialog" aria-label="Connection details">
          <div class="row"><span>Status</span><strong>{label()}</strong></div>
          <div class="row"><span>Latency</span><span>{connectionState().latencyMs ?? "—"} ms</span></div>
          <div class="row"><span>Last sync</span><span>{connectionState().lastSyncISO ? new Date(connectionState().lastSyncISO).toLocaleTimeString() : "—"}</span></div>
        </div>
      </Show>
    </div>
  );
}