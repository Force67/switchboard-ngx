import { createSignal, createEffect, onMount, createMemo } from "solid-js";
import "./theme.css";
import "./app.css";
import Sidebar from "./components/Sidebar";
import MainArea from "./components/MainArea";
import { apiService } from "./api";
import { initializeFromAPI } from "./components/sidebarStore";
import { useSocket } from "./hooks/useSocket";

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
  id?: string;
  chat_id?: string;
  user_id?: number;
  role: "user" | "assistant" | "system";
  content: string;
  model?: string;
  usage?: TokenUsage;
  reasoning?: string[];
  timestamp?: string;
  message_type?: string;
}

interface Chat {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  folderId?: string;
  updatedAt?: number;
  isGroup?: boolean;
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
  pricing?: {
    input?: number;
    output?: number;
  };
  supports_reasoning?: boolean;
  supports_images?: boolean;
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
  // Temporary test session for development
  const testSession: SessionData = {
    token: "test-token",
    user: {
      id: "test-user",
      email: "test@example.com",
      display_name: "Test User",
    },
    expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
  };

  const [session, setSession] = createSignal<SessionData | null>(
    testSession, // loadStoredSession(),
  );
  const [prompt, setPrompt] = createSignal("");
  const [attachedImages, setAttachedImages] = createSignal<File[]>([]);
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
  const [modelPickerOpen, setModelPickerOpen] = createSignal(false);

  // WebSocket integration
  const socket = useSocket();

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

  const newChat = async (folderId?: string, isGroup: boolean = false) => {
    const activeSession = session();
    if (!activeSession) return;

    try {
      const apiChat = await apiService.createChat(activeSession.token, {
        title: isGroup ? "New Group Chat" : "New Chat",
        messages: [],
        folder_id: folderId,
        is_group: isGroup,
      });

      const newChatObj: Chat = {
        id: apiChat.public_id,
        public_id: apiChat.public_id,
        title: apiChat.title,
        messages: [],
        createdAt: new Date(apiChat.created_at),
        folderId,
        updatedAt: apiChat.updated_at,
        isGroup: apiChat.is_group,
      };

      setChats(prev => [newChatObj, ...prev]);
      setCurrentChatId(apiChat.public_id);
      setPrompt("");
      setError(null);
    } catch (error) {
      console.error("Failed to create chat", error);
      setError("Failed to create new chat");
    }
  };

  const newGroupChat = async (folderId?: string) => {
    await newChat(folderId, true);
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

    // Check WebSocket connection
    if (socket.state.status !== 'connected') {
      setError("Real-time connection not available. Please check your connection.");
      return;
    }

    setLoading(true);
    setError(null);

    // Add user message to current chat immediately for UI responsiveness
    const updatedChat = chats().find(c => c.id === currentId);
    if (!updatedChat) return;

    const userMessage: Message = {
      role: "user",
      content: trimmedPrompt,
      user_id: 1, // Current user
      timestamp: new Date().toISOString(),
    };

    const newMessages = [...updatedChat.messages, userMessage];
    const newTitle = updatedChat.messages.length === 0 ? trimmedPrompt.slice(0, 30) + (trimmedPrompt.length > 30 ? "..." : "") : updatedChat.title;

    setChats(prev => prev.map(chat =>
      chat.id === currentId
        ? {
            ...chat,
            messages: newMessages,
            title: newTitle
          }
        : chat
    ));
    setPrompt("");
    setAttachedImages([]);

    try {
      // Send message via WebSocket
      socket.sendMessage(currentId, trimmedPrompt);

      // Note: Assistant response will come via WebSocket and be handled by the effect above

    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to send message");
    } finally {
      setLoading(false);
    }
  };

  // WebSocket event handling
  createEffect(() => {
    const message = socket.state.lastMessage;
    if (!message) return;

    const currentId = currentChatId();
    if (!currentId) return;

    if (message.type === 'message' && message.chat_id === currentId) {
      // Add new message to current chat
      const newMessage: Message = {
        id: message.message_id,
        chat_id: message.chat_id,
        user_id: message.user_id,
        role: message.user_id === 1 ? 'user' : 'assistant', // Assuming user_id 1 is current user
        content: message.content,
        timestamp: message.timestamp,
        message_type: message.message_type,
      };

      setChats(prev => prev.map(chat =>
        chat.id === currentId
          ? {
              ...chat,
              messages: [...chat.messages, newMessage]
            }
          : chat
      ));
    }
  });

  createEffect(() => {
    const current = session();
    if (current) {
      void loadModels(current);
      void loadChatsAndFolders(current.token);
      // Connect WebSocket with auth token
      socket.connect();
    } else {
      setModels([]);
      setSelectedModel("");
      setChats([]);
      setCurrentChatId(null);
      // Disconnect WebSocket
      socket.disconnect();
    }
  });

  const loadChatsAndFolders = async (token: string) => {
    try {
      // Initialize sidebar with folders
      await initializeFromAPI(token);

      // Load chats
      const apiChats = await apiService.listChats(token);
      const frontendChats: Chat[] = apiChats.map(apiChat => {
        let messages: Message[] = [];
        try {
          messages = JSON.parse(apiChat.messages);
        } catch (e) {
          console.error("Failed to parse chat messages", e);
        }

        return {
          id: apiChat.public_id,
          public_id: apiChat.public_id,
          title: apiChat.title,
          messages,
          createdAt: new Date(apiChat.created_at),
          folderId: undefined, // Will be resolved by sidebar
          updatedAt: apiChat.updated_at,
          isGroup: apiChat.is_group,
        };
      });

      setChats(frontendChats);

      // Create initial chat if none exist
      if (frontendChats.length === 0) {
        await newChat();
      } else {
        // Select the most recent chat
        setCurrentChatId(frontendChats[0].id);
      }
    } catch (error) {
      console.error("Failed to load chats and folders", error);
      // Fallback to creating a new chat
      if (chats().length === 0) {
        await newChat();
      }
    }
  };

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
        onNewGroupChat={newGroupChat}
        onSelectChat={selectChat}
      />
        <MainArea
          prompt={prompt}
          setPrompt={setPrompt}
          attachedImages={attachedImages}
          setAttachedImages={setAttachedImages}
          selectedModel={selectedModel}
          setSelectedModel={setSelectedModel}
          models={models}
          modelsLoading={modelsLoading}
          modelsError={modelsError}
          loading={loading}
          error={error}
          modelPickerOpen={modelPickerOpen}
          setModelPickerOpen={setModelPickerOpen}
          session={session}
          connectionStatus={createMemo(() => ({
            status: socket.state.status,
            error: socket.state.error || undefined
          }))}
          currentMessages={createMemo(() => {
            const currentId = currentChatId();
            const currentChat = chats().find(c => c.id === currentId);
            return currentChat ? currentChat.messages : [];
          })}
          currentChat={createMemo(() => {
            const currentId = currentChatId();
            return chats().find(c => c.id === currentId) || null;
          })}
          onSend={handleSubmit}
          onLogout={logout}
        />
    </div>
  );
}
