import { Accessor, createSignal, onMount, onCleanup } from "solid-js";

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

export default function UserPill(props: Props) {
  const [showMenu, setShowMenu] = createSignal(false);
  const [menuPosition, setMenuPosition] = createSignal({ x: 0, y: 0 });

  const handleContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    setMenuPosition({ x: event.clientX, y: event.clientY });
    setShowMenu(true);
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (showMenu() && !(event.target as Element).closest('.user-pill-menu')) {
      setShowMenu(false);
    }
  };

  const handleLogout = () => {
    setShowMenu(false);
    props.onLogout();
  };

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener('click', handleClickOutside);
  });

  const session = props.session();
  if (!session) return null;

  const displayName = session.user.display_name || session.user.email || "User";

  return (
    <>
      <div
        class="user-pill"
        onContextMenu={handleContextMenu}
        title="Right-click to logout"
      >
        <svg viewBox="0 0 16 16" width="12" height="12">
          <circle cx="8" cy="8" r="4" fill="currentColor" opacity="0.7" />
          <path d="M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8S12.4 0 8 0zm0 12c-2.2 0-4-1.8-4-4s1.8-4 4-4 4 1.8 4 4-1.8 4-4 4z" fill="currentColor" />
        </svg>
        <span class="user-name">{displayName}</span>
      </div>

      {showMenu() && (
        <div
          class="user-pill-menu"
          style={{
            position: 'fixed',
            left: `${menuPosition().x}px`,
            top: `${menuPosition().y}px`,
            zIndex: 1000
          }}
        >
          <button class="menu-item" onClick={handleLogout}>
            <svg viewBox="0 0 16 16" width="14" height="14">
              <path d="M2 4v8c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2z" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M6 8h4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            Logout
          </button>
        </div>
      )}
    </>
  );
}