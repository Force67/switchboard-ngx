import { createSignal, createEffect, onMount, createMemo } from "solid-js";
import "./theme.css";
import "./app.css";
import Sidebar from "./components/Sidebar";
import MainArea from "./components/MainArea";

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

interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  usage?: TokenUsage;
  reasoning?: string[];
}

interface Chat {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
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

export default function App() {
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
  const [chats, setChats] = createSignal<Chat[]>([]);
  const [currentChatId, setCurrentChatId] = createSignal<string | null>(null);
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
    setChats([]);
    setCurrentChatId(null);
    setPrompt("");
  };

  const newChat = () => {
    const chatId = `chat_${Date.now()}`;
    const newChatObj: Chat = {
      id: chatId,
      title: "New Chat",
      messages: [],
      createdAt: new Date(),
    };
    setChats(prev => [newChatObj, ...prev]);
    setCurrentChatId(chatId);
    setPrompt("");
    setError(null);
  };

  const selectChat = (chatId: string) => {
    setCurrentChatId(chatId);
    setPrompt("");
    setError(null);
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

    const currentId = currentChatId();
    if (!currentId) {
      newChat(); // Create new chat if none selected
      return handleSubmit(event); // Retry
    }

    setLoading(true);
    setError(null);

    // Add user message to current chat
    setChats(prev => prev.map(chat =>
      chat.id === currentId
        ? {
            ...chat,
            messages: [...chat.messages, { role: "user", content: trimmedPrompt }],
            title: chat.messages.length === 0 ? trimmedPrompt.slice(0, 30) + (trimmedPrompt.length > 30 ? "..." : "") : chat.title
          }
        : chat
    ));
    setPrompt("");

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
      // Add assistant message to current chat
      setChats(prev => prev.map(chat =>
        chat.id === currentId
          ? {
              ...chat,
              messages: [...chat.messages, {
                role: "assistant",
                content: data.content,
                model: data.model,
                usage: data.usage,
                reasoning: data.reasoning
              }]
            }
          : chat
      ));
    } catch (err) {
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

  createEffect(() => {
    if (session() && chats().length === 0) {
      newChat();
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
    <div class="app">
      <Sidebar
        session={session}
        chats={chats}
        currentChatId={currentChatId}
        onLogin={beginGithubLogin}
        onLogout={logout}
        onNewChat={newChat}
        onSelectChat={selectChat}
      />
      <MainArea
        prompt={prompt}
        setPrompt={setPrompt}
        selectedModel={selectedModel}
        setSelectedModel={setSelectedModel}
        models={models}
        modelsLoading={modelsLoading}
        modelsError={modelsError}
        loading={loading}
        error={error}
        currentMessages={createMemo(() => {
          const currentId = currentChatId();
          const currentChat = chats().find(c => c.id === currentId);
          return currentChat ? currentChat.messages : [];
        })}
        onSend={handleSubmit}
      />
    </div>
  );
}
