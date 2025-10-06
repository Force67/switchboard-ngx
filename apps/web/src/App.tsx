import type { Component } from "solid-js";
import {
  For,
  Show,
  createEffect,
  createSignal,
  onMount,
} from "solid-js";

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:7070";
const DEFAULT_MODEL = import.meta.env.VITE_DEFAULT_MODEL ?? "";
const GITHUB_REDIRECT_PATH =
  import.meta.env.VITE_GITHUB_REDIRECT_PATH ?? "/auth/callback";
const SESSION_KEY = "switchboard.session";

interface UserProfile {
  id: string;
  email?: string | null;
  display_name?: string | null;
}

interface SessionData {
  token: string;
  user: UserProfile;
  expires_at: string;
}

interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

interface ChatResponse {
  model: string;
  content: string;
  usage?: TokenUsage;
  reasoning?: string[];
}

interface ErrorResponse {
  error: string;
}

interface ModelsResponse {
  models: ModelOption[];
}

interface ModelOption {
  id: string;
  label: string;
  description?: string | null;
}

const loadStoredSession = (): SessionData | null => {
  if (typeof window === "undefined") {
    return null;
  }

  const json = window.localStorage.getItem(SESSION_KEY);
  if (!json) {
    return null;
  }

  try {
    const parsed = JSON.parse(json) as SessionData;
    if (new Date(parsed.expires_at).getTime() <= Date.now()) {
      window.localStorage.removeItem(SESSION_KEY);
      return null;
    }
    return parsed;
  } catch (error) {
    console.error("Failed to parse session", error);
    window.localStorage.removeItem(SESSION_KEY);
    return null;
  }
};

const App: Component = () => {
  const [session, setSession] = createSignal<SessionData | null>(
    loadStoredSession(),
  );
  const [prompt, setPrompt] = createSignal("");
  const [selectedModel, setSelectedModel] = createSignal<string>(DEFAULT_MODEL);
  const [models, setModels] = createSignal<ModelOption[]>([]);
  const [modelsLoading, setModelsLoading] = createSignal(false);
  const [modelsError, setModelsError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [response, setResponse] = createSignal<ChatResponse | null>(null);
  const [authenticating, setAuthenticating] = createSignal(false);
  const [authError, setAuthError] = createSignal<string | null>(null);

  const redirectUri = () => `${window.location.origin}${GITHUB_REDIRECT_PATH}`;

  const persistSession = (value: SessionData | null) => {
    setSession(value);
    if (typeof window === "undefined") {
      return;
    }
    if (value) {
      window.localStorage.setItem(SESSION_KEY, JSON.stringify(value));
    } else {
      window.localStorage.removeItem(SESSION_KEY);
    }
  };

  const finalizeGithubLogin = async (code: string, state: string) => {
    setAuthError(null);
    try {
      const response = await fetch(`${API_BASE}/api/auth/github/callback`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          code,
          state,
          redirect_uri: redirectUri(),
        }),
      });

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as
          | ErrorResponse
          | null;
        throw new Error(body?.error ?? response.statusText);
      }

      const data = (await response.json()) as SessionData;
      persistSession(data);
      await loadModels(data);
      window.history.replaceState(null, "", "/");
      setAuthError(null);
      setError(null);
    } catch (err) {
      persistSession(null);
      window.history.replaceState(null, "", "/");
      setAuthError(
        err instanceof Error
          ? err.message
          : "Unable to complete GitHub login",
      );
    } finally {
      setAuthenticating(false);
    }
  };

  const beginGithubLogin = async () => {
    setAuthError(null);
    setAuthenticating(true);
    try {
      const response = await fetch(
        `${API_BASE}/api/auth/github/login?redirect_uri=${encodeURIComponent(redirectUri())}`,
      );

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as
          | ErrorResponse
          | null;
        throw new Error(body?.error ?? response.statusText);
      }

      const { authorize_url } = (await response.json()) as {
        authorize_url: string;
      };

      window.location.href = authorize_url;
    } catch (err) {
      setAuthError(
        err instanceof Error ? err.message : "Unable to start GitHub login",
      );
      setAuthenticating(false);
    }
  };

  const loadModels = async (activeSession: SessionData) => {
    setModelsLoading(true);
    setModelsError(null);
    try {
      const response = await fetch(`${API_BASE}/api/models`, {
        headers: {
          Authorization: `Bearer ${activeSession.token}`,
        },
      });

      if (response.status === 401) {
        persistSession(null);
        throw new Error("Session expired. Please sign in again.");
      }

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as
          | ErrorResponse
          | null;
        throw new Error(body?.error ?? response.statusText);
      }

      const data = (await response.json()) as ModelsResponse;
      setModels(data.models);

      const preferred = (() => {
        const current = selectedModel().trim();
        if (current && data.models.some((model) => model.id === current)) {
          return current;
        }
        if (
          DEFAULT_MODEL &&
          data.models.some((model) => model.id === DEFAULT_MODEL)
        ) {
          return DEFAULT_MODEL;
        }
        return data.models[0]?.id ?? "";
      })();

      setSelectedModel(preferred);
    } catch (err) {
      setModels([]);
      setSelectedModel("");
      setModelsError(
        err instanceof Error ? err.message : "Unable to load models",
      );
    } finally {
      setModelsLoading(false);
    }
  };

  const logout = () => {
    persistSession(null);
    setModels([]);
    setSelectedModel("");
    setResponse(null);
    setPrompt("");
  };

  const handleSubmit = async (event: Event) => {
    event.preventDefault();

    const activeSession = session();
    if (!activeSession) {
      setError("Please sign in with GitHub first.");
      return;
    }

    const trimmedPrompt = prompt().trim();
    if (!trimmedPrompt) {
      setError("Please enter a prompt first.");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const payload: Record<string, string> = { prompt: trimmedPrompt };
      const model = selectedModel().trim();
      if (model) {
        payload.model = model;
      }

      const response = await fetch(`${API_BASE}/api/chat`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${activeSession.token}`,
        },
        body: JSON.stringify(payload),
      });

      if (response.status === 401) {
        logout();
        throw new Error("Session expired. Please sign in again.");
      }

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as
          | ErrorResponse
          | null;
        throw new Error(body?.error ?? response.statusText);
      }

      const data = (await response.json()) as ChatResponse;
      setResponse(data);
      setPrompt("");
    } catch (err) {
      setResponse(null);
      setError(err instanceof Error ? err.message : "Unexpected error");
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    const current = session();
    if (current) {
      void loadModels(current);
    } else {
      setModels([]);
      setSelectedModel("");
    }
  });

  onMount(() => {
    const url = new URL(window.location.href);
    const code = url.searchParams.get("code");
    const state = url.searchParams.get("state");
    const oauthError = url.searchParams.get("error");

    if (oauthError) {
      setAuthError(`GitHub: ${oauthError}`);
      window.history.replaceState(null, "", "/");
      return;
    }

    if (code && state) {
      setAuthenticating(true);
      void finalizeGithubLogin(code, state);
    }
  });

  return (
    <main class="app">
      <header>
        <h1>Switchboard NGX Playground</h1>
        <p>
          Sign in with GitHub to send prompts through OpenRouter. Configure
          <code> VITE_API_BASE</code> if the backend runs on a different host.
        </p>
      </header>

      <Show
        when={session()}
        fallback={
          <section class="card login">
            <h2>GitHub Required</h2>
            <p>Authenticate with GitHub SSO before accessing the playground.</p>
            <button
              type="button"
              disabled={authenticating()}
              onClick={beginGithubLogin}
            >
              {authenticating() ? "Completing login..." : "Continue with GitHub"}
            </button>
            <Show when={authError()}>
              {(message) => <div class="error">{message()}</div>}
            </Show>
          </section>
        }
      >
        {(activeSession) => (
          <>
            <section class="card session">
              <div class="session-heading">
                <div>
                  <h2>Signed in</h2>
                  <p>
                    {activeSession().user.display_name ?? "GitHub user"}
                    <Show when={activeSession().user.email}>
                      {(email) => <span class="muted"> ({email()})</span>}
                    </Show>
                  </p>
                </div>
                <button type="button" class="outline" onClick={logout}>
                  Log out
                </button>
              </div>
              <p class="session-meta">
                Token expires {new Date(activeSession().expires_at).toLocaleString()}
              </p>
            </section>

            <form class="card" onSubmit={handleSubmit}>
              <div class="field">
                <span>Model</span>
                <select
                  value={selectedModel()}
                  onChange={(event) => setSelectedModel(event.currentTarget.value)}
                  disabled={modelsLoading() || models().length === 0}
                >
                  <For each={models()}>
                    {(model) => (
                      <option value={model.id}>
                        {model.label}
                        {model.description ? ` — ${model.description}` : ""}
                      </option>
                    )}
                  </For>
                </select>
                <Show when={modelsLoading()}>
                  <span class="hint-text">Fetching models...</span>
                </Show>
                <Show when={modelsError()}>
                  {(message) => <div class="error">{message()}</div>}
                </Show>
              </div>

              <label class="field">
                <span>Prompt</span>
                <textarea
                  placeholder="Ask the LLM something..."
                  rows={6}
                  value={prompt()}
                  onInput={(event) => setPrompt(event.currentTarget.value)}
                />
              </label>

              <div class="actions">
                <button type="submit" disabled={loading()}>
                  {loading() ? "Sending..." : "Send"}
                </button>
              </div>
            </form>

            <Show when={error()}>
              {(message) => <div class="error">{message()}</div>}
            </Show>

            <Show when={response()}>
              {(result) => (
                <section class="card output">
                  <header>
                    <h2>Response</h2>
                    <small>
                      Model: <code>{result().model}</code>
                    </small>
                  </header>
                  <pre>{result().content}</pre>

                  <Show when={result().reasoning && result().reasoning!.length > 0}>
                    <div class="reasoning">
                      <h3>Reasoning</h3>
                      <ol>
                        {result()
                          .reasoning!.map((step) => (
                            <li>{step}</li>
                          ))}
                      </ol>
                    </div>
                  </Show>

                  <Show when={result().usage}>
                    {(usage) => (
                      <footer class="meta">
                        <span>{usage().prompt_tokens} prompt tokens</span>
                        <span>{usage().completion_tokens} completion tokens</span>
                        <span>{usage().total_tokens} total tokens</span>
                      </footer>
                    )}
                  </Show>
                </section>
              )}
            </Show>
          </>
        )}
      </Show>

      <footer class="hint">
        <span>Backend:</span>
        <code>{API_BASE}</code>
      </footer>
    </main>
  );
};

export default App;
