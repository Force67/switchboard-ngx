import { Accessor, createSignal, onMount, onCleanup } from "solid-js";
import type { SessionData } from "../types/session";

interface Props {
  session: Accessor<SessionData | null>;
  onLogout: () => void;
  onEditProfile: () => void;
}

export default function UserPill(props: Props) {
  const [showMenu, setShowMenu] = createSignal(false);
  const [menuPosition, setMenuPosition] = createSignal({ x: 0, y: 0 });
  let pillRef: HTMLDivElement | undefined;

  const updateMenuPosition = () => {
    if (!pillRef) return;
    const rect = pillRef.getBoundingClientRect();
    setMenuPosition({
      x: rect.left,
      y: rect.bottom + 6,
    });
  };

  const toggleMenu = (event: MouseEvent) => {
    event.preventDefault();
    updateMenuPosition();
    setShowMenu((prev) => !prev);
  };

  const closeMenu = () => setShowMenu(false);

  const handleClickOutside = (event: MouseEvent) => {
    if (
      showMenu() &&
      !(event.target as Element).closest(".user-pill-menu") &&
      !(event.target as Element).closest(".user-pill")
    ) {
      closeMenu();
    }
  };

  const handleResize = () => {
    if (showMenu()) {
      updateMenuPosition();
    }
  };

  const handleLogout = () => {
    closeMenu();
    props.onLogout();
  };

  const handleEditProfile = () => {
    closeMenu();
    props.onEditProfile();
  };

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
    window.addEventListener("resize", handleResize);
  });

  onCleanup(() => {
    document.removeEventListener("click", handleClickOutside);
    window.removeEventListener("resize", handleResize);
  });

  const session = props.session();
  if (!session) return null;

  const displayName =
    session.user.display_name ||
    session.user.username ||
    session.user.email ||
    "User";
  const rawUsername = session.user.username;
  const handle =
    rawUsername &&
    rawUsername.toLowerCase() !== displayName.toLowerCase()
      ? rawUsername
      : null;
  const avatarUrl = session.user.avatar_url ?? undefined;
  const userBio = session.user.bio ?? "";

  return (
    <>
      <div
        ref={pillRef}
        class="user-pill"
        onClick={toggleMenu}
        title="Manage profile"
      >
        {avatarUrl ? (
          <img
            src={avatarUrl}
            alt="Profile avatar"
            class="user-pill-avatar"
          />
        ) : (
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <circle cx="8" cy="6" r="3" fill="currentColor" opacity="0.85" />
            <path
              d="M2.5 14c0-3 2.5-5 5.5-5s5.5 2 5.5 5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
          </svg>
        )}
        <div class="user-pill-text">
          <span class="user-name">{displayName}</span>
          {handle && (
            <span class="user-handle">@{handle}</span>
          )}
        </div>
      </div>

      {showMenu() && (
        <div
          class="user-pill-menu"
          style={`position: fixed; left: ${menuPosition().x}px; top: ${menuPosition().y}px; z-index: 1000;`}
        >
          <div class="user-pill-menu-header">
            {avatarUrl ? (
              <img
                src={avatarUrl}
                alt="Profile avatar"
                class="user-pill-menu-avatar"
              />
            ) : (
              <div class="user-pill-menu-avatar fallback">
                <svg
                  viewBox="0 0 16 16"
                  width="18"
                  height="18"
                  aria-hidden="true"
                >
                  <circle cx="8" cy="6" r="3.2" fill="currentColor" />
                  <path
                    d="M2.5 14.5c0-3.3 2.7-5.5 5.5-5.5s5.5 2.2 5.5 5.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.3"
                    stroke-linecap="round"
                  />
                </svg>
              </div>
            )}
            <div class="user-pill-menu-meta">
              <span class="user-pill-menu-name">{displayName}</span>
              {rawUsername && (
                <span class="user-pill-menu-username">@{rawUsername}</span>
              )}
            </div>
          </div>
          {userBio && <p class="user-pill-menu-bio">{userBio}</p>}
          <div class="user-pill-menu-divider" />
          <button class="menu-item" onClick={handleEditProfile}>
            <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
              <path
                d="M11 2l3 3L6 13H3v-3L11 2z"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linejoin="round"
              />
              <path
                d="M9.5 3.5l3 3"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
            Edit Profile
          </button>
          <button class="menu-item" onClick={handleLogout}>
            <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
              <path
                d="M2 4v8c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2z"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
              />
              <path
                d="M6 8h4"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
            Logout
          </button>
        </div>
      )}
    </>
  );
}
