import { Accessor } from "solid-js";
import OnlineIndicator from "./OnlineIndicator";
import UserPill from "./UserPill";

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
}

export default function TopRightControls(props: Props) {
  return (
    <div class="top-bar">
      <div class="top-right">
        <button class="icircle">
          <svg viewBox="0 0 14 14">
            <path d="M7 1L5 7H8L6 13L9 7H6L7 1Z" />
          </svg>
        </button>
        <button class="icircle">
          <svg viewBox="0 0 14 14">
            <path d="M4.5 2.5a.5.5 0 0 0-.5.5v7a.5.5 0 0 0 .5.5h5a.5.5 0 0 0 .5-.5V3a.5.5 0 0 0-.5-.5h-2a.5.5 0 0 1 0-1h2A1.5 1.5 0 0 1 10.5 3v7a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 2.5 10V3A1.5 1.5 0 0 1 4 1.5h2a.5.5 0 0 0 0-1h-2z" />
          </svg>
        </button>
        <button class="icircle">
          <svg viewBox="0 0 14 14">
            <circle cx="7" cy="7" r="3" />
            <path d="M7 0v2M7 12v2M0 7h2M12 7h2M1.5 1.5l1.5 1.5M11 11l1.5 1.5M1.5 12.5l1.5-1.5M11 3l1.5-1.5" />
          </svg>
        </button>
        <OnlineIndicator />
        <UserPill session={props.session} onLogout={props.onLogout} />
      </div>
    </div>
  );
}