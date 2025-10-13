import { Accessor } from "solid-js";
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
}

export default function TopRightControls(props: Props) {
  return (
    <div class="top-bar">
      <div class="top-left">
        {/* Left side - can be used for additional controls */}
      </div>
      <div class="top-center">
        <h1 class="app-title">Switchboard NGX</h1>
      </div>
      <div class="top-right">
        <ThemeToggle />
        {props.connectionStatus && (
          <WebSocketStatusIndicator status={props.connectionStatus} />
        )}
        <OnlineIndicator />
        <UserPill session={props.session} onLogout={props.onLogout} />
      </div>
    </div>
  );
}