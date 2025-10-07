import { Accessor } from "solid-js";

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
  onLogin: () => void;
  onLogout: () => void;
}

export default function SidebarFooter(props: Props) {
  const session = props.session();

  if (session) {
    return null; // User info is now in top right
  }

  return (
    <div class="sidebar-footer" onClick={props.onLogin}>
      <svg viewBox="0 0 16 16">
        <path d="M8.5 2.5a.5.5 0 0 0-1 0v5.793L5.354 6.146a.5.5 0 1 0-.708.708l3 3a.5.5 0 0 0 .708 0l3-3a.5.5 0 0-.708-.708L8.5 8.293V2.5z" />
      </svg>
      Login
    </div>
  );
}