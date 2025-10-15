import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
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

  const session = () => props.session();
  const user = createMemo(() => session()?.user ?? null);
  const displayName = createMemo(() => {
    const current = user();
    if (!current) {
      return "User";
    }
    return (
      current.display_name ||
      current.username ||
      current.email ||
      "User"
    );
  });
  const rawUsername = createMemo(() => user()?.username ?? null);
  const handle = createMemo(() => {
    const username = rawUsername();
    const name = displayName();
    if (!username) {
      return null;
    }
    return username.toLowerCase() !== name.toLowerCase() ? username : null;
  });
  const avatarUrl = createMemo(() => user()?.avatar_url ?? undefined);
  const userBio = createMemo(() => user()?.bio ?? "");

  createEffect(() => {
    if (!user()) {
      setShowMenu(false);
    }
  });

  return (
    <Show when={user()} fallback={null}>
      {() => (
        <>
          <div
            ref={pillRef}
            class="user-pill"
            onClick={toggleMenu}
            title="Manage profile"
          >
            <Show
              when={avatarUrl()}
              fallback={
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
              }
            >
              {(avatar) => (
                <img
                  src={avatar()}
                  alt="Profile avatar"
                  class="user-pill-avatar"
                />
              )}
            </Show>
            <div class="user-pill-text">
              <span class="user-name">{displayName()}</span>
              <Show when={handle()}>
                {(value) => <span class="user-handle">@{value()}</span>}
              </Show>
            </div>
          </div>

          {showMenu() && (
            <div
              class="user-pill-menu"
              style={`position: fixed; left: ${menuPosition().x}px; top: ${menuPosition().y}px; z-index: 1000;`}
            >
              <div class="user-pill-menu-header">
                <Show
                  when={avatarUrl()}
                  fallback={
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
                  }
                >
                  {(avatar) => (
                    <img
                      src={avatar()}
                      alt="Profile avatar"
                      class="user-pill-menu-avatar"
                    />
                  )}
                </Show>
                <div class="user-pill-menu-meta">
                  <span class="user-pill-menu-name">{displayName()}</span>
                  <Show when={rawUsername()}>
                    {(username) => (
                      <span class="user-pill-menu-username">@{username()}</span>
                    )}
                  </Show>
                </div>
              </div>
              <Show when={userBio()}>
                {(bio) => <p class="user-pill-menu-bio">{bio()}</p>}
              </Show>
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
      )}
    </Show>
  );
}
