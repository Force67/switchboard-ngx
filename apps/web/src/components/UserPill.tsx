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
  console.log('UserPill - session:', session);

  // Always render for debugging
  const displayName = session?.user.display_name || session?.user.email || "Debug User";

  console.log('UserPill - rendering with displayName:', displayName);

  return (
    <>
      <div
        class="user-pill"
        onContextMenu={handleContextMenu}
        title="Right-click for options"
        style={{ border: '2px solid red !important' }}
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
              <path d="M10 12.5a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 .5.5v2a.5.5 0 0 0 1 0v-2A1.5 1.5 0 0 0 9.5 2h-8A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h8a1.5 1.5 0 0 0 1.5-1.5v-2a.5.5 0 0 0-1 0v2z"/>
              <path d="M15.854 8.354a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708.708L14.293 7.5H5.5a.5.5 0 0 0 0 1h8.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3z"/>
            </svg>
            Logout
          </button>
        </div>
      )}
    </>
  );
}