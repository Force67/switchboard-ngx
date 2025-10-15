import { Accessor, Show, createMemo } from "solid-js";
import type { SessionData } from "../types/session";

interface Props {
  session: Accessor<SessionData | null>;
  onLogin: () => void;
  onLogout: () => void;
  onEditProfile: () => void;
}

export default function SidebarFooter(props: Props) {
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
  const avatarUrl = createMemo(() => user()?.avatar_url ?? undefined);
  const usernameForBadge = createMemo(() => {
    const current = user();
    if (!current?.username) {
      return null;
    }
    return current.username !== displayName() ? current.username : null;
  });
  const emailForBadge = createMemo(() => {
    const current = user();
    if (!current?.email) {
      return null;
    }
    return current.email !== displayName() ? current.email : null;
  });

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
          return (
            <div class="sidebar-account">
              <div class="sidebar-account-card">
                <Show
                  when={avatarUrl()}
                  fallback={
                    <div class="sidebar-account-avatar fallback">
                      {displayName().slice(0, 1).toUpperCase()}
                    </div>
                  }
                >
                  {(src) => (
                    <img
                      src={src()}
                      alt="Profile"
                      class="sidebar-account-avatar"
                    />
                  )}
                </Show>
                <div class="sidebar-account-meta">
                  <span class="sidebar-account-name">{displayName()}</span>
                  <Show when={usernameForBadge()}>
                    {(username) => (
                      <span class="sidebar-account-username">
                        @{username()}
                      </span>
                    )}
                  </Show>
                  <Show when={emailForBadge()}>
                    {(email) => (
                      <span class="sidebar-account-email">{email()}</span>
                    )}
                  </Show>
                  <span class="sidebar-account-email">
                    ID: {active().user.id}
                  </span>
                </div>
              </div>

              <Show when={usernameForBadge()}>
                {(username) => (
                  <div class="sidebar-account-provider">
                    <span class="provider-label">GitHub</span>
                    <strong>@{username()}</strong>
                  </div>
                )}
              </Show>

              <Show when={active().user.bio}>
                {(bio) => <p class="sidebar-account-bio">{bio()}</p>}
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
