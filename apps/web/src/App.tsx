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

// Auto-fetch dev token for development
const fetchDevSession = async (): Promise<SessionData | null> => {
  try {
    const response = await fetch(`${API_BASE}/api/auth/dev/token`);
    if (!response.ok) return null;

    const data = await response.json();
    return {
      token: data.token,
      user: {
        id: data.user.id,
        email: data.user.email,
        display_name: data.user.display_name,
      },
      expires_at: data.expires_at,
    };
  } catch (error) {
    console.error("Failed to fetch dev token:", error);
    return null;
  }
};

export default function App() {
  const [session, setSession] = createSignal<SessionData | null>(null);
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
  const socket = useSocket(() => session()?.token || null);

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

    // Handle case where folderId might be a click event object
    const validFolderId = (folderId && typeof folderId === 'string') ? folderId : undefined;

    try {
      const apiChat = await apiService.createChat(activeSession.token, {
        title: isGroup ? "New Group Chat" : "New Chat",
        messages: [],
        folder_id: validFolderId,
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

    // Check WebSocket connection and subscription
    const connectionStatus = socket.state().status;
    const subscribedId = currentSubscription();

    console.log("🔍 Pre-send check:", {
      connectionStatus,
      currentId,
      subscribedId,
      isSubscribed: currentId === subscribedId
    });

    if (connectionStatus !== 'connected') {
      setError("Real-time connection not available. Please check your connection.");
      return;
    }

    if (currentId !== subscribedId) {
      setError("Not subscribed to this chat yet. Please wait a moment and try again.");
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
    const socketState = socket.state();
    console.log("🔥 WebSocket effect triggered, socketState:", socketState);
    const message = socketState.lastMessage;
    if (!message) {
      console.log("❌ No message to process");
      return;
    }

    console.log("🔍 Processing WebSocket message:", message);
    const currentId = currentChatId();
    console.log("📱 Current chat ID:", currentId);
    console.log("🔗 Message chat ID:", message.chat_id);
    console.log("✅ Chat IDs match:", message.chat_id === currentId);
    console.log("📊 Current chats count:", chats().length);

    if (!currentId) {
      console.log("❌ No current chat ID, skipping message");
      return;
    }

    if (message.type === 'message') {
      console.log("📨 Message type is 'message'");
      if (message.chat_id === currentId) {
        console.log("✅ Message chat ID matches current chat ID - processing message");
      // Check if this message already exists in the chat (user messages are added immediately)
      const currentChat = chats().find(c => c.id === currentId);
      console.log("🔍 Current chat found:", !!currentChat);
      console.log("📊 Current chat messages count:", currentChat?.messages.length || 0);

      const messageExists = currentChat?.messages.some(m => m.id === message.message_id);
      console.log("🔍 Message exists in chat:", messageExists);
      console.log("🔍 Looking for message ID:", message.message_id);
      console.log("🔍 Current chat message IDs:", currentChat?.messages.map(m => m.id));

      if (messageExists) {
        // This is a user message that was already added to UI, skip
        console.log("⏭️ Message already exists, skipping (user message echo)");
        return;
      }

      // Check if this looks like a user message by comparing with the last user message
      const lastUserMessage = currentChat?.messages
        .filter(m => m.role === 'user')
        .pop();

      if (lastUserMessage && lastUserMessage.content === message.content) {
        console.log("⏭️ Skipping user message echo (content matches last user message)");
        return;
      }

      console.log("🤖 New message detected, adding to chat...");
      // All messages received via WebSocket that aren't already in the chat should be assistant responses
      // User messages are added immediately to UI when sent, so WebSocket messages are always assistant responses

      const newMessage: Message = {
        id: message.message_id,
        chat_id: message.chat_id,
        user_id: message.user_id,
        role: 'assistant',
        content: message.content,
        timestamp: message.timestamp,
        message_type: message.message_type,
      };

      console.log("📝 New message to add:", newMessage);

      setChats(prev => {
        const updated = prev.map(chat =>
          chat.id === currentId
            ? {
                ...chat,
                messages: [...chat.messages, newMessage]
              }
            : chat
        );
        console.log("🔄 Updated chats:", updated);
        console.log("📊 Chat with new message:", updated.find(c => c.id === currentId)?.messages);
        return updated;
      });
      } else {
        console.log("❌ Message chat ID does NOT match current chat ID:", {
          messageChatId: message.chat_id,
          currentId,
          chatIdsMatch: message.chat_id === currentId
        });
      }
    } else {
      console.log("❌ Message type is not 'message':", {
        messageType: message.type,
        messageChatId: message.chat_id,
        currentId,
        isCorrectType: message.type === 'message',
        isCorrectChat: message.chat_id === currentId
      });
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

  // Track current subscription to avoid spam
  const [currentSubscription, setCurrentSubscription] = createSignal<string | null>(null);

  // Subscribe to current chat via WebSocket
  createEffect(() => {
    const currentId = currentChatId();
    const connectionStatus = socket.state().status;
    const subscribedId = currentSubscription();

    console.log("🔍 Subscription check:", {
      currentId,
      connectionStatus,
      subscribedId,
      shouldSubscribe: currentId && connectionStatus === 'connected' && currentId !== subscribedId
    });

    // Only subscribe when we have a chat, WebSocket is connected, and we're not already subscribed
    if (currentId && connectionStatus === 'connected' && currentId !== subscribedId) {
      console.log("📡 Subscribing to chat:", currentId);
      socket.subscribe(currentId);
      setCurrentSubscription(currentId);
    } else if (!currentId && subscribedId) {
      // Clear subscription if no chat selected
      console.log("🗑️ Clearing subscription");
      setCurrentSubscription(null);
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
    } else {
      // For development: auto-fetch dev token if no OAuth flow
      void fetchDevSession().then((devSession) => {
        if (devSession) {
          setSession(devSession);
          persistSession(devSession);
        }
      });
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
          connectionStatus={createMemo(() => {
            const state = socket.state();
            console.log('WebSocket state:', state);
            return {
              status: state.status,
              error: state.error || undefined
            };
          })}
          currentMessages={createMemo(() => {
            const currentId = currentChatId();
            const currentChat = chats().find(c => c.id === currentId);
            const messages = currentChat ? currentChat.messages : [];
            console.log("🔄 currentMessages memo recalculated:", { currentId, messagesCount: messages.length, messages });
            return messages;
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
