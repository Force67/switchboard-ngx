import { Accessor, Show } from "solid-js";
import type { SessionData } from "../types/session";

interface Props {
  session: Accessor<SessionData | null>;
  onLogin: () => void;
  onLogout: () => void;
  onEditProfile: () => void;
}

export default function SidebarFooter(props: Props) {
  const session = () => props.session();

  return (
    <>
      <Show when={!session()}>
        <div class="sidebar-footer" onClick={props.onLogin}>
          <svg viewBox="0 0 16 16">
            <path d="M8.5 2.5a.5.5 0 0 0-1 0v5.793L5.354 6.146a.5.5 0 1 0-.708.708l3 3a.5.5 0 0 0 .708 0l3-3a.5.5 0 0-.708-.708L8.5 8.293V2.5z" />
          </svg>
          Login
        </div>
      </Show>

      <Show when={session()}>
        {(active) => {
          const user = active().user;
          const displayName =
            user.display_name || user.username || user.email || "User";
          const avatarUrl = user.avatar_url || undefined;
          return (
            <div class="sidebar-account">
              <div class="sidebar-account-card">
                {avatarUrl ? (
                  <img src={avatarUrl} alt="Profile" class="sidebar-account-avatar" />
                ) : (
                  <div class="sidebar-account-avatar fallback">
                    {displayName.slice(0, 1).toUpperCase()}
                  </div>
                )}
              <div class="sidebar-account-meta">
                <span class="sidebar-account-name">{displayName}</span>
                <Show when={user.username && user.username !== displayName}>
                  <span class="sidebar-account-username">@{user.username}</span>
                </Show>
                <Show when={user.email && user.email !== displayName}>
                  <span class="sidebar-account-email">{user.email}</span>
                </Show>
                <span class="sidebar-account-email">ID: {active().user.id}</span>
              </div>
            </div>

              <Show when={user.username}>
                <div class="sidebar-account-provider">
                  <span class="provider-label">GitHub</span>
                  <strong>@{user.username}</strong>
                </div>
              </Show>

              <Show when={user.bio}>
                <p class="sidebar-account-bio">{user.bio}</p>
              </Show>

              <div class="sidebar-account-actions">
                <button type="button" class="sidebar-account-btn primary" onClick={props.onEditProfile}>
                  Edit profile
                </button>
                <button type="button" class="sidebar-account-btn" onClick={props.onLogout}>
                  Logout
                </button>
              </div>
            </div>
          );
        }}
      </Show>
    </>
  );
}
