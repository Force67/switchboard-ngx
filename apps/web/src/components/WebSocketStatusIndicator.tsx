import { Accessor } from "solid-js";

interface Props {
  status: Accessor<{ status: string; error?: string }>;
}

export default function WebSocketStatusIndicator(props: Props) {
  const getStatusColor = () => {
    const status = props.status().status;
    switch (status) {
      case "connected": return "#4CAF50"; // green
      case "connecting": return "#FF9800"; // orange
      case "error": return "#F44336"; // red
      case "disconnected": return "#9E9E9E"; // grey
      default: return "#9E9E9E"; // grey for unknown
    }
  };

  const getStatusText = () => {
    const status = props.status().status;
    const error = props.status().error;

    switch (status) {
      case "connected": return "WS Connected";
      case "connecting": return "WS Connecting...";
      case "error": return `WS Error: ${error || "Unknown"}`;
      case "disconnected": return "WS Disconnected";
      default: return "WS Unknown";
    }
  };

  const getStatusIcon = () => {
    const status = props.status().status;

    switch (status) {
      case "connected":
        return (
          <svg viewBox="0 0 12 12" fill="currentColor">
            <circle cx="6" cy="6" r="3" />
            <circle cx="6" cy="6" r="5" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3" />
          </svg>
        );
      case "connecting":
        return (
          <svg viewBox="0 0 12 12" fill="currentColor">
            <circle cx="6" cy="6" r="3" opacity="0.3" />
            <path d="M6 1v1M6 10v1M1 6h1M10 6h1" stroke="currentColor" stroke-width="1" opacity="0.5" />
          </svg>
        );
      case "error":
        return (
          <svg viewBox="0 0 12 12" fill="currentColor">
            <circle cx="6" cy="6" r="4" fill="none" stroke="currentColor" stroke-width="1" />
            <path d="M4 4l4 4M8 4l-4 4" stroke="currentColor" stroke-width="1" />
          </svg>
        );
      case "disconnected":
        return (
          <svg viewBox="0 0 12 12" fill="currentColor">
            <circle cx="6" cy="6" r="4" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3" />
            <path d="M3 6h6" stroke="currentColor" stroke-width="1" opacity="0.5" />
          </svg>
        );
      default:
        return (
          <svg viewBox="0 0 12 12" fill="currentColor">
            <circle cx="6" cy="6" r="4" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3" />
            <path d="M6 3v3l2 2" stroke="currentColor" stroke-width="1" opacity="0.5" />
          </svg>
        );
    }
  };

  return (
    <div
      class="websocket-status-indicator"
      style={`
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        border-radius: 12px;
        background: ${getStatusColor()}20;
        color: ${getStatusColor()};
        font-size: 11px;
        font-weight: 500;
        border: 1px solid ${getStatusColor()}40;
        transition: all 0.2s ease;
      `}
      title={getStatusText()}
    >
      <span style="width: 12px; height: 12px; display: flex; align-items: center; justify-content: center;">
        {getStatusIcon()}
      </span>
      <span style="text-transform: uppercase; letter-spacing: 0.5px;">
        {props.status().status === "connected" ? "LIVE" :
         props.status().status === "connecting" ? "..." :
         props.status().status === "error" ? "ERR" :
         props.status().status === "disconnected" ? "OFF" : "???"}
      </span>
    </div>
  );
}