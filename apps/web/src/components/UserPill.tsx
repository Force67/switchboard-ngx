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

type ThemeMode = "light" | "dark" | "auto";

interface Props {
  session: Accessor<SessionData | null>;
  onLogout: () => void;
  /** Optional: force theme; default "auto" uses prefers-color-scheme */
  theme?: ThemeMode;
}

export default function UserPill(props: Props) {
  const [showMenu, setShowMenu] = createSignal(false);
  const [showProfile, setShowProfile] = createSignal(false);
  const [autoDark, setAutoDark] = createSignal(false);
  let pillRef: HTMLDivElement | undefined;
  let mql: MediaQueryList | undefined;

  // Theme resolution
  const resolvedTheme = createMemo<"light" | "dark">(() => {
    if (props.theme === "light") return "light";
    if (props.theme === "dark") return "dark";
    return autoDark() ? "dark" : "light";
  });

  onMount(() => {
    // auto: track system dark mode
    mql = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (mql) {
      setAutoDark(mql.matches);
      const handler = (e: MediaQueryListEvent) => setAutoDark(e.matches);
      mql.addEventListener?.("change", handler);
      // cleanup listener
      onCleanup(() => mql?.removeEventListener?.("change", handler));
    }
    document.addEventListener("click", handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  // Design tokens
  const tokens = createMemo(() => {
    const dark = resolvedTheme() === "dark";
    return {
      // surfaces
      bgBase: dark ? "#0b0f16" : "#ffffff",
      bgElevated: dark ? "#161b22" : "#ffffff",
      pillBg: dark ? "#0f1420" : "#ffffff",
      pillBorder: dark ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.10)",
      menuBg: dark ? "#0f1420" : "#ffffff",
      menuBorder: dark ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.08)",
      overlay: dark ? "rgba(0,0,0,0.6)" : "rgba(0,0,0,0.4)",
      // text
      text: dark ? "#e5e7eb" : "#111827",
      textMuted: dark ? "#9ca3af" : "#6b7280",
      // effects
      shadow: dark ? "0 12px 28px rgba(0,0,0,0.45)" : "0 12px 28px rgba(0,0,0,0.12)",
      ring: dark ? "1px solid rgba(255,255,255,0.08)" : "1px solid rgba(0,0,0,0.08)",
      // accents
      brand: "#8b5cf6",
      danger: "#ef4444",
      // interactive
      hoverBg: dark ? "rgba(255,255,255,0.05)" : "rgba(0,0,0,0.04)"
    };
  });

  const handleClick = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    setShowMenu(v => !v);
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (showMenu() && !(event.target as Element).closest(".user-pill-container")) {
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

  const sessionData = createMemo(() => props.session());
  const displayName = createMemo(() => {
    const s = sessionData();
    return s?.user.display_name || s?.user.email || "User";
  });

  return (
    <Show when={sessionData()}>
      <div class="user-pill-container" style={{ position: "relative", display: "inline-block" }}>
        {/* Pill */}
        <div
          ref={pillRef}
          class="user-pill"
          onClick={handleClick}
          title="Account"
          style={{
            display: "inline-flex",
            "align-items": "center",
            gap: "8px",
            padding: "6px 10px",
            "border-radius": "9999px",
            border: tokens().ring,
            "background-color": tokens().pillBg,
            color: tokens().text,
            "box-shadow": tokens().shadow,
            cursor: "pointer",
            "user-select": "none",
            "font-size": "14px",
            "font-weight": 500,
            "z-index": 10,
            transition: "background-color 120ms ease, border-color 120ms ease"
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = tokens().hoverBg)}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = tokens().pillBg)}
        >
          <svg viewBox="0 0 16 16" width="14" height="14" style={{ opacity: 0.85 }}>
            <circle cx="8" cy="8" r="4" fill="currentColor" opacity="0.6" />
            <path
              d="M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8S12.4 0 8 0zm0 12c-2.2 0-4-1.8-4-4s1.8-4 4-4 4 1.8 4 4-1.8 4-4 4z"
              fill="currentColor"
            />
          </svg>
          <span
            class="user-name"
            style={{ "max-width": "160px", overflow: "hidden", "text-overflow": "ellipsis", "white-space": "nowrap" }}
          >
            {displayName()}
          </span>
          <svg viewBox="0 0 20 20" width="14" height="14" aria-hidden="true" style={{ opacity: 0.7 }}>
            <path d="M5.5 7.5L10 12l4.5-4.5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" />
          </svg>
        </div>

        {/* Menu */}
        {showMenu() && pillRef && (
          <div
            class="user-pill-menu"
            style={{
              position: "absolute",
              top: `${pillRef.offsetHeight + 6}px`,
              right: "0",
              "z-index": 20,
              width: "220px",
              padding: "8px",
              "background-color": tokens().menuBg,
              border: tokens().ring,
              "border-radius": "12px",
              "box-shadow": tokens().shadow,
              color: tokens().text
            }}
          >
            <button
              class="menu-item"
              onClick={handleViewProfile}
              style={{
                width: "100%",
                display: "flex",
                gap: "8px",
                "align-items": "center",
                padding: "10px",
                "border-radius": "8px",
                border: "none",
                background: "transparent",
                cursor: "pointer",
                "font-size": "14px",
                color: tokens().text
              }}
              onMouseDown={(e) => e.preventDefault()}
              onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = tokens().hoverBg)}
              onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
            >
              <svg viewBox="0 0 16 16" width="16" height="16">
                <path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM2 13c0-2.5 3-4 6-4s6 1.5 6 4c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2z" fill="currentColor" />
              </svg>
              View profile
            </button>

            <button
              class="menu-item"
              onClick={handleLogout}
              style={{
                width: "100%",
                display: "flex",
                gap: "8px",
                "align-items": "center",
                padding: "10px",
                "border-radius": "8px",
                border: "none",
                background: "transparent",
                cursor: "pointer",
                "font-size": "14px",
                color: tokens().danger
              }}
              onMouseDown={(e) => e.preventDefault()}
              onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = tokens().hoverBg)}
              onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
            >
              <svg viewBox="0 0 16 16" width="16" height="16">
                <path d="M10 12.5a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 .5.5v2a.5.5 0 0 0 1 0v-2A1.5 1.5 0 0 0 9.5 2h-8A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h8a1.5 1.5 0 0 0 1.5-1.5v-2a.5.5 0 0 0-1 0v2z" fill="currentColor" />
                <path d="M15.854 8.354a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708.708L14.293 7.5H5.5a.5.5 0 0 0 0 1h8.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3z" fill="currentColor" />
              </svg>
              Logout
            </button>
          </div>
        )}

        {/* Profile Modal (centered; theme-aware) */}
        {showProfile() && sessionData() && (
          <div
            role="dialog"
            aria-modal="true"
            style={{
              position: "fixed",
              inset: "0",
              "background-color": tokens().overlay,
              display: "grid",
              "place-items": "center",
              "z-index": 50
            }}
            onClick={() => setShowProfile(false)}
          >
            <div
              style={{
                width: "min(92vw, 480px)",
                "background-color": tokens().bgElevated,
                color: tokens().text,
                padding: "28px",
                "border-radius": "16px",
                position: "relative",
                "box-shadow": tokens().shadow,
                border: tokens().ring
              }}
              onClick={(e) => e.stopPropagation()}
            >
              <button
                aria-label="Close"
                onClick={() => setShowProfile(false)}
                style={{
                  position: "absolute",
                  top: "12px",
                  right: "12px",
                  width: "32px",
                  height: "32px",
                  "border-radius": "50%",
                  border: tokens().ring,
                  background: "transparent",
                  color: tokens().text,
                  cursor: "pointer"
                }}
              >
                ×
              </button>

              {/* Header */}
              <div style={{ display: "flex", "align-items": "center", gap: "16px", "margin-bottom": "20px" }}>
                <div
                  style={{
                    width: "72px",
                    height: "72px",
                    "border-radius": "50%",
                    background: tokens().brand,
                    display: "flex",
                    "align-items": "center",
                    "justify-content": "center",
                    "font-size": "28px",
                    "font-weight": 700,
                    border: "3px solid rgba(255,255,255,0.9)",
                    color: "#fff"
                  }}
                >
                  {displayName().charAt(0).toUpperCase()}
                </div>
                <div>
                  <h2 style={{ margin: 0, "font-size": "22px" }}>{displayName()}</h2>
                  <p style={{ margin: "6px 0 0 0", color: tokens().textMuted }}>
                    {sessionData()!.user.email || "No email"}
                  </p>
                </div>
              </div>

              {/* Details */}
              <div style={{ display: "grid", gap: "12px" }}>
                <div
                  style={{
                    padding: "12px",
                    "background-color": resolvedTheme() === "dark" ? "#0b1220" : "#f9fafb",
                    border: tokens().ring,
                    "border-radius": "10px"
                  }}
                >
                  <div style={{ "font-size": "12px", color: tokens().textMuted, "margin-bottom": "6px" }}>User ID</div>
                  <div
                    style={{
                      "font-family":
                        "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace",
                      "font-size": "13px",
                      padding: "8px",
                      "border-radius": "6px",
                      "background-color": resolvedTheme() === "dark" ? "#0b0f16" : "#ffffff",
                      border: tokens().ring
                    }}
                  >
                    {sessionData()!.user.id}
                  </div>
                </div>

                <div
                  style={{
                    padding: "12px",
                    "background-color": resolvedTheme() === "dark" ? "#0b1220" : "#f9fafb",
                    border: tokens().ring,
                    "border-radius": "10px"
                  }}
                >
                  <div style={{ "font-size": "12px", color: tokens().textMuted, "margin-bottom": "6px" }}>
                    Session Expires
                  </div>
                  <div style={{ "font-size": "14px" }}>
                    {new Date(sessionData()!.expires_at).toLocaleString()}
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div style={{ display: "flex", "justify-content": "flex-end", "margin-top": "18px", gap: "10px" }}>
                <button
                  onClick={() => setShowProfile(false)}
                  style={{
                    padding: "10px 16px",
                    "border-radius": "10px",
                    border: tokens().ring,
                    background: "transparent",
                    color: tokens().text,
                    cursor: "pointer"
                  }}
                >
                  Close
                </button>
                <button
                  onClick={handleLogout}
                  style={{
                    padding: "10px 16px",
                    "border-radius": "10px",
                    border: "none",
                    background: tokens().danger,
                    color: "#fff",
                    cursor: "pointer",
                    "box-shadow": "0 6px 16px rgba(239,68,68,0.4)"
                  }}
                >
                  Logout
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </Show>
  );
}
