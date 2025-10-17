import { Accessor, createSignal, onMount, onCleanup, createMemo, Show } from "solid-js";

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
  console.log('🔥 UserPill component rendering!', props);
  const [showMenu, setShowMenu] = createSignal(false);
  const [showProfile, setShowProfile] = createSignal(false);
  let pillRef: HTMLDivElement | undefined;

  const handleClick = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    console.log('🔥 Pill clicked! Current showMenu:', showMenu());
    setShowMenu(!showMenu());
    console.log('🔥 After toggle showMenu:', showMenu());
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (showMenu() && !(event.target as Element).closest('.user-pill-container')) {
      console.log('🔥 Click outside, closing menu');
      setShowMenu(false);
    }
  };

  const handleViewProfile = () => {
    setShowMenu(false);
    setShowProfile(true);
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

  const sessionData = createMemo(() => props.session());

  const displayName = createMemo(() => {
    const session = sessionData();
    if (!session) return null;
    return session.user.display_name || session.user.email || "User";
  });

  return (
    <Show when={sessionData()}>
      <div class="user-pill-container" style={{ position: 'relative', display: 'inline-block' }}>
        <div
          ref={pillRef}
          class="user-pill"
          onClick={handleClick}
          title="Click for options"
          style={{
            border: '2px solid red !important',
            backgroundColor: 'yellow !important',
            color: 'black !important',
            pointerEvents: 'auto',
            userSelect: 'none',
            cursor: 'pointer',
            zIndex: 9999
          }}
        >
        <svg viewBox="0 0 16 16" width="12" height="12">
          <circle cx="8" cy="8" r="4" fill="currentColor" opacity="0.7" />
          <path d="M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8S12.4 0 8 0zm0 12c-2.2 0-4-1.8-4-4s1.8-4 4-4 4 1.8 4 4-1.8 4-4 4z" fill="currentColor" />
        </svg>
        <span class="user-name">{displayName()}</span>
      </div>

      {showMenu() && pillRef && (
        <div
          class="user-pill-menu"
          style={{
            position: 'absolute',
            top: `${pillRef.offsetHeight + 4}px`,
            right: '0',
            zIndex: 1000,
            minWidth: '180px'
          }}
        >
          <button class="menu-item" onClick={handleViewProfile}>
            <svg viewBox="0 0 16 16" width="14" height="14">
              <path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm2-3a2 2 0 1 1-4 0 2 2 0 0 1 4 0zm4 8c0 1-1 2-2 2H4c-1 0-2-1-2-2 0-1.5.5-3 2.5-4.5S7.5 7 8 7s1.5.5 3.5 2S14 11.5 14 13z"/>
            </svg>
            View Profile
          </button>
          <button class="menu-item" onClick={handleLogout}>
            <svg viewBox="0 0 16 16" width="14" height="14">
              <path d="M10 12.5a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 .5.5v2a.5.5 0 0 0 1 0v-2A1.5 1.5 0 0 0 9.5 2h-8A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h8a1.5 1.5 0 0 0 1.5-1.5v-2a.5.5 0 0 0-1 0v2z"/>
              <path d="M15.854 8.354a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708.708L14.293 7.5H5.5a.5.5 0 0 0 0 1h8.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3z"/>
            </svg>
            Logout
          </button>
        </div>
      )}

      {/* Profile Modal */}
      {showProfile() && (
        <div
          style={{
            position: 'fixed',
            top: '0',
            left: '0',
            right: '0',
            bottom: '0',
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 2000
          }}
          onClick={() => setShowProfile(false)}
        >
          <div
            style={{
              backgroundColor: 'var(--bg-2)',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: '8px',
              padding: '24px',
              minWidth: '400px',
              maxWidth: '500px'
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div style={{
              display: 'flex',
              alignItems: 'center',
              marginBottom: '20px',
              paddingBottom: '16px',
              borderBottom: '1px solid rgba(255,255,255,0.1)'
            }}>
              <div style={{
                width: '64px',
                height: '64px',
                borderRadius: '50%',
                backgroundColor: 'var(--accent)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                marginRight: '16px',
                color: 'var(--bg-2)',
                fontSize: '24px',
                fontWeight: 'bold'
              }}>
                {displayName.charAt(0).toUpperCase()}
              </div>
              <div>
                <h2 style={{
                  margin: '0',
                  fontSize: '20px',
                  fontWeight: '600',
                  color: 'var(--text-0)'
                }}>
                  {displayName}
                </h2>
                <p style={{
                  margin: '4px 0 0 0',
                  fontSize: '14px',
                  color: 'var(--text-1)'
                }}>
                  {session.user.email || 'No email'}
                </p>
              </div>
            </div>

            <div style={{
              display: 'flex',
              flexDirection: 'column',
              gap: '12px',
              marginBottom: '24px'
            }}>
              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                padding: '8px 0',
                borderBottom: '1px solid rgba(255,255,255,0.1)'
              }}>
                <span style={{ color: 'var(--text-1)', fontSize: '14px' }}>User ID</span>
                <span style={{
                  color: 'var(--text-0)',
                  fontSize: '14px',
                  fontFamily: 'monospace',
                  backgroundColor: 'var(--bg-3)',
                  padding: '2px 6px',
                  borderRadius: '4px'
                }}>
                  {session.user.id}
                </span>
              </div>

              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                padding: '8px 0'
              }}>
                <span style={{ color: 'var(--text-1)', fontSize: '14px' }}>Session Expires</span>
                <span style={{ color: 'var(--text-0)', fontSize: '14px' }}>
                  {new Date(session.expires_at).toLocaleDateString()}
                </span>
              </div>
            </div>

            <div style={{
              display: 'flex',
              justifyContent: 'flex-end'
            }}>
              <button
                onClick={() => setShowProfile(false)}
                style={{
                  backgroundColor: 'var(--accent)',
                  color: 'var(--bg-2)',
                  border: 'none',
                  borderRadius: '6px',
                  padding: '8px 16px',
                  fontSize: '14px',
                  fontWeight: '500',
                  cursor: 'pointer'
                }}
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
    </Show>
  );
}