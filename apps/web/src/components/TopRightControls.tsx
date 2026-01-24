import { Accessor, Show } from "solid-js";
import OnlineIndicator from "./OnlineIndicator";
import UserPill from "./UserPill";
import WebSocketStatusIndicator from "./WebSocketStatusIndicator";
import ThemeToggle from "./ThemeToggle";

interface SessionData {
  token: string;
  user: {
    id: string;
    email?: string | null;
    display_name?: string | null;
  };
  expires_at: string;
}

interface Props {
  session: Accessor<SessionData | null>;
  onLogout: () => void;
  connectionStatus?: Accessor<{ status: string; error?: string }>;
  propertiesSidebarOpen?: Accessor<boolean>;
  onToggleProperties?: () => void;
  hasActiveProperties?: Accessor<boolean>;
  onOpenSidebar?: () => void;
}

export default function TopRightControls(props: Props) {
  return (
    <div class="top-bar">
      <div class="top-left">
        <button
          class="mobile-menu-btn"
          onClick={() => props.onOpenSidebar?.()}
          aria-label="Open sidebar menu"
        >
          <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" fill="none" stroke-width="2">
            <path d="M3 12h18M3 6h18M3 18h18" />
          </svg>
        </button>
      </div>
      <div class="top-center">
        <h1 class="app-title">Switchboard NGX</h1>
      </div>
      <div class="top-right">
        <ThemeToggle />
        <Show when={props.onToggleProperties}>
          <button
            class={`properties-toggle-btn ${props.propertiesSidebarOpen?.() ? "active" : ""} ${props.hasActiveProperties?.() ? "has-active" : ""}`}
            onClick={() => props.onToggleProperties?.()}
            aria-label="Toggle chat settings"
            title="Chat Settings (Shift+S)"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M12 15a3 3 0 100-6 3 3 0 000 6z" />
              <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
            </svg>
            <span class="active-dot" />
          </button>
        </Show>
        {props.connectionStatus && (
          <WebSocketStatusIndicator status={props.connectionStatus} />
        )}
        <OnlineIndicator />
        <UserPill session={props.session} onLogout={props.onLogout} />
      </div>
    </div>
  );
}