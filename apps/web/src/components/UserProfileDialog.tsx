import {
  Accessor,
  Show,
  createEffect,
  createSignal,
  onCleanup,
} from "solid-js";
import { apiService } from "../api";
import type { SessionData, SessionUser } from "../types/session";

interface Props {
  open: Accessor<boolean>;
  session: Accessor<SessionData | null>;
  onClose: () => void;
  onUpdated: (user: SessionUser) => void;
}

const sanitizeField = (value: string): string | null => {
  const trimmed = value.trim();
  return trimmed.length === 0 ? null : trimmed;
};

export default function UserProfileDialog(props: Props) {
  const [displayName, setDisplayName] = createSignal("");
  const [bio, setBio] = createSignal("");
  const [avatarUrl, setAvatarUrl] = createSignal("");
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const sessionUser = () => props.session()?.user;

  createEffect(() => {
    if (!props.open()) {
      return;
    }

    const current = props.session();
    if (!current) {
      return;
    }

    const user = current.user;
    setDisplayName(user.display_name ?? "");
    setBio(user.bio ?? "");
    setAvatarUrl(user.avatar_url ?? "");
    setError(null);
  });

  createEffect(() => {
    if (!props.open()) {
      return;
    }

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        props.onClose();
      }
    };

    window.addEventListener("keydown", handleKeydown);
    onCleanup(() => window.removeEventListener("keydown", handleKeydown));
  });

  const handleSubmit = async (event: Event) => {
    event.preventDefault();
    const current = props.session();
    if (!current) {
      return;
    }

    setSaving(true);
    setError(null);

    try {
      const cleanedDisplayName =
        sanitizeField(displayName()) ??
        sanitizeField(current.user.username ?? "") ??
        null;
      const payload = {
        display_name: cleanedDisplayName,
        bio: sanitizeField(bio()),
        avatar_url: sanitizeField(avatarUrl()),
      };

      const updatedUser = await apiService.updateCurrentUser(
        current.token,
        payload,
      );
      props.onUpdated(updatedUser);
      props.onClose();
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to update profile. Please try again.",
      );
    } finally {
      setSaving(false);
    }
  };

  const handleBackdropClick = (event: MouseEvent) => {
    if (event.target === event.currentTarget) {
      props.onClose();
    }
  };

  return (
    <Show when={props.open()}>
      <div class="profile-dialog-backdrop" onClick={handleBackdropClick}>
        <div class="profile-dialog" onClick={(event) => event.stopPropagation()}>
          <form class="profile-dialog-content" onSubmit={handleSubmit}>
            <h2>Profile</h2>
            <p class="profile-dialog-subtitle">
              Update how other people see you across the workspace.
            </p>
            <Show when={sessionUser()}>
              {(active) => (
                <div class="profile-oauth-summary">
                  <span class="summary-label">Signed in with GitHub</span>
                  <span class="summary-value">
                    @{active().username ?? active().email ?? "unknown"}
                  </span>
                  <span class="summary-subtext">Account ID: {active().id}</span>
                  <Show when={active().email}>
                    {(email) => (
                      <span class="summary-subtext">Email: {email()}</span>
                    )}
                  </Show>
                </div>
              )}
            </Show>

            <label class="profile-field">
              <span class="profile-field-label">Display name</span>
              <input
                type="text"
                value={displayName()}
                onInput={(event) => setDisplayName(event.currentTarget.value)}
                placeholder="Your name"
                maxLength={64}
              />
            </label>

            <label class="profile-field">
              <span class="profile-field-label">Username</span>
              <input
                type="text"
                value={sessionUser()?.username ?? ""}
                readOnly
                placeholder="username"
                maxLength={64}
              />
              <span class="profile-hint">
                Usernames come from your sign-in provider and cannot be changed.
              </span>
            </label>

            <label class="profile-field">
              <span class="profile-field-label">Bio</span>
              <textarea
                value={bio()}
                onInput={(event) => setBio(event.currentTarget.value)}
                placeholder="Tell others a little about yourself"
                maxLength={512}
                rows={4}
              />
            </label>

            <label class="profile-field">
              <span class="profile-field-label">Avatar URL</span>
              <input
                type="url"
                value={avatarUrl()}
                onInput={(event) => setAvatarUrl(event.currentTarget.value)}
                placeholder="https://example.com/avatar.png"
              />
              <span class="profile-hint">
                Use a direct link to an image. Leave blank to use the default
                avatar.
              </span>
            </label>

            {error() && <div class="profile-error">{error()}</div>}

            <div class="profile-actions">
              <button
                type="button"
                class="profile-secondary"
                onClick={props.onClose}
                disabled={saving()}
              >
                Cancel
              </button>
              <button type="submit" class="profile-primary" disabled={saving()}>
                {saving() ? "Saving..." : "Save changes"}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Show>
  );
}
