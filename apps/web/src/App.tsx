import { createSignal, createEffect, onMount, createMemo } from "solid-js";
import "./theme.css";
import "./app.css";
import { ThemeProvider } from "./contexts/ThemeContext";
import Sidebar from "./components/Sidebar";
import MainArea from "./components/MainArea";
import { apiService } from "./api";
import type { ApiChat } from "./api";
import {
  actions,
  addChatToSidebar,
  initializeFromAPI,
  removeChatFromSidebar,
  sidebarState,
} from "./components/sidebarStore";
import type { Actions } from "./components/sidebarTypes";
import type { SidebarBootstrapData } from "./components/sidebarStore";
import { useSocket } from "./hooks/useSocket";
import type { Chat, Message, TokenUsage } from "./types/chat";

const DEFAULT_API_BASE =
  typeof window !== "undefined" ? window.location.origin : "http://localhost:7070";
const API_BASE = import.meta.env.VITE_API_BASE ?? DEFAULT_API_BASE;
const DEFAULT_MODEL = import.meta.env.VITE_DEFAULT_MODEL ?? "";
const GITHUB_REDIRECT_PATH =
  import.meta.env.VITE_GITHUB_REDIRECT_PATH ?? "/auth/callback";
const SESSION_KEY = "switchboard.session";
const ENABLE_DEV_LOGIN = import.meta.env.VITE_ENABLE_DEV_LOGIN === "false";

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
  console.log('🚀 App render called');
  const [session, setSession] = createSignal<SessionData | null>(null);
  const [prompt, setPrompt] = createSignal("");
  const [attachedImages, setAttachedImages] = createSignal<File[]>([]);
  const [selectedModels, setSelectedModels] = createSignal<string[]>(
    DEFAULT_MODEL ? [DEFAULT_MODEL] : []
  );
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
  const [testLoading, setTestLoading] = createSignal(false);
  const [modelStatuses, setModelStatuses] = createSignal<Record<string, "idle" | "pending">>({});

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

      const available = data.models;
      const currentSelection = Array.from(
        new Set(selectedModels().filter((id) => available.some((model) => model.id === id)))
      );

      let nextSelection = currentSelection;
      if (nextSelection.length === 0) {
        const fallback =
          (DEFAULT_MODEL && available.find((model) => model.id === DEFAULT_MODEL)?.id) ??
          available[0]?.id;
        nextSelection = fallback ? [fallback] : [];
      }

      setSelectedModels(nextSelection);
      setModelStatuses(prev => {
        const next: Record<string, "idle" | "pending"> = {};
        nextSelection.forEach(id => {
          next[id] = prev[id] ?? "idle";
        });
        return next;
      });
    } catch (err) {
      setModels([]);
      setSelectedModels([]);
      setModelStatuses({});
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
    setSelectedModels([]);
    setModelStatuses({});
    setChats([]);
    setCurrentChatId(null);
    setPrompt("");
  };

  const newChat = async (folderId?: string, isGroup: boolean = false) => {
    const activeSession = session();
    if (!activeSession) return;

    // Handle case where folderId might be a click event object
    const validFolderId = folderId && typeof folderId === "string" ? folderId : undefined;

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
        folderId: validFolderId,
        updatedAt: apiChat.updated_at,
        isGroup: apiChat.is_group,
      };

      setChats(prev => [newChatObj, ...prev.filter(chat => chat.id !== newChatObj.id)]);
      addChatToSidebar(apiChat.public_id, validFolderId);
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

  const renameChat = async (chatId: string, title: string) => {
    const activeSession = session();
    if (!activeSession) return;

    try {
      await apiService.updateChat(activeSession.token, chatId, { title });
      setChats(prev =>
        prev.map(chat => (chat.id === chatId ? { ...chat, title } : chat)),
      );
      setError(null);
    } catch (error) {
      console.error("Failed to rename chat", error);
      setError("Failed to rename chat");
    }
  };

  const deleteChat = async (chatId: string) => {
    const activeSession = session();
    if (!activeSession) return;

    try {
      await apiService.deleteChat(activeSession.token, chatId);
      removeChatFromSidebar(chatId);
      const updatedChats = chats().filter(chat => chat.id !== chatId);
      setChats(updatedChats);

      if (currentChatId() === chatId) {
        setCurrentChatId(updatedChats[0]?.id ?? null);
        setPrompt("");
      }

      setError(null);
    } catch (error) {
      console.error("Failed to delete chat", error);
      setError("Failed to delete chat");
    }
  };

  const deleteFolder = async (folderId: string) => {
    const activeSession = session();
    if (!activeSession) return;

    const snapshot = sidebarState();
    const folderIds: string[] = [];
    const collect = (id: string) => {
      folderIds.push(id);
      const subs = snapshot.subfolderOrder[id] || [];
      subs.forEach(collect);
    };
    collect(folderId);

    const chatIdsToRemove = new Set<string>();
    for (const id of folderIds) {
      const chatsInFolder = snapshot.chatOrderByFolder[id] || [];
      chatsInFolder.forEach(chatId => chatIdsToRemove.add(chatId));
    }

    try {
      for (const chatId of chatIdsToRemove) {
        await apiService.deleteChat(activeSession.token, chatId);
        removeChatFromSidebar(chatId);
      }

      await actions.deleteFolder(folderId, "delete-all");

      if (chatIdsToRemove.size > 0) {
        const updatedChats = chats().filter(chat => !chatIdsToRemove.has(chat.id));
        setChats(updatedChats);
        const currentId = currentChatId();
        if (currentId && chatIdsToRemove.has(currentId)) {
          setCurrentChatId(updatedChats[0]?.id ?? null);
          setPrompt("");
        }
      }

      setError(null);
    } catch (error) {
      console.error("Failed to delete folder", error);
      setError("Failed to delete folder");
    }
  };

  const selectChat = (chatId: string) => {
    setCurrentChatId(chatId);
    setPrompt("");
    setError(null);
  };

  const resolveModelMentions = (input: string) => {
    const available = models();
    if (!input) {
      return { cleaned: input, mentions: [] as string[] };
    }

    const register = (map: Map<string, string>, key: string, id: string) => {
      const trimmed = key.trim();
      if (trimmed && !map.has(trimmed)) {
        map.set(trimmed, id);
      }
    };

    const generateKeys = (raw: string) => {
      const variants = new Set<string>();
      const base = raw.toLowerCase().trim();
      if (!base) return variants;

      const withoutParens = base.replace(/\([^)]*\)/g, "");
      const collapsedWhitespace = base.replace(/\s+/g, "");
      const collapsedWhitespaceNoParens = withoutParens.replace(/\s+/g, "");
      const softClean = base.replace(/[^\w:.\-+]/g, "");
      const softCleanNoParens = withoutParens.replace(/[^\w:.\-+]/g, "");
      const hardClean = base.replace(/[^a-z0-9]/g, "");
      const hardCleanNoParens = withoutParens.replace(/[^a-z0-9]/g, "");

      [
        base,
        withoutParens,
        collapsedWhitespace,
        collapsedWhitespaceNoParens,
        softClean,
        softCleanNoParens,
        hardClean,
        hardCleanNoParens,
      ]
        .map((value) => value.trim())
        .filter(Boolean)
        .forEach((value) => variants.add(value));

      return variants;
    };

    const lookup = new Map<string, string>();
    available.forEach((model) => {
      generateKeys(model.id).forEach((key) => register(lookup, key, model.id));
      if (model.label) {
        generateKeys(model.label).forEach((key) => register(lookup, key, model.id));
      }
    });

    const mentionRegex = /@([^\s]+)/g;
    const mentions: string[] = [];
    const removals: Array<{ start: number; end: number }> = [];

    let match: RegExpExecArray | null;
    while ((match = mentionRegex.exec(input)) !== null) {
      const rawToken = match[1];
      const tokenWithoutTrailing = rawToken.replace(/[.,;:!?]+$/, "");
      const candidateKeys = generateKeys(tokenWithoutTrailing);
      let resolved: string | undefined;
      for (const key of candidateKeys) {
        resolved = lookup.get(key);
        if (resolved) break;
      }

      if (resolved) {
        mentions.push(resolved);
      }

      removals.push({ start: match.index, end: match.index + match[0].length });
    }

    if (removals.length === 0) {
      return { cleaned: input, mentions: [] as string[] };
    }

    removals.sort((a, b) => a.start - b.start);
    let cursor = 0;
    let result = "";
    for (const removal of removals) {
      result += input.slice(cursor, removal.start);
      cursor = removal.end;
      while (cursor < input.length && input[cursor] === " ") {
        cursor += 1;
      }
    }
    result += input.slice(cursor);

    const seen = new Set<string>();
    const orderedMentions = mentions.filter((id) => {
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    });

    return {
      cleaned: result,
      mentions: orderedMentions,
    };
  };

  createEffect(() => {
    const selected = selectedModels();
    setModelStatuses(prev => {
      const next: Record<string, "idle" | "pending"> = {};
      selected.forEach(id => {
        next[id] = prev[id] ?? "idle";
      });
      return next;
    });
  });

  const handleSubmit = async (event: Event) => {
    event.preventDefault();

    const activeSession = session();
    if (!activeSession) {
      setError("Please sign in with GitHub first.");
      return;
    }

    const rawPrompt = prompt();
    const { cleaned: cleanedPrompt, mentions } = resolveModelMentions(rawPrompt);
    const trimmedPrompt = cleanedPrompt.trim();
    if (!trimmedPrompt) {
      setError("Please enter a prompt first.");
      return;
    }

    const baseSelection = Array.from(
      new Set(selectedModels().filter((id) => id && id.trim().length > 0)),
    );
    const mentionSelection = mentions;
    let targetModels = mentionSelection.length > 0 ? mentionSelection : baseSelection;

    if (targetModels.length === 0) {
      const availableModels = models();
      const fallbackId = (() => {
        if (
          DEFAULT_MODEL &&
          availableModels.some((model) => model.id === DEFAULT_MODEL)
        ) {
          return DEFAULT_MODEL;
        }
        return availableModels[0]?.id;
      })();

      if (fallbackId) {
        targetModels = [fallbackId];
      }
    }

    if (targetModels.length === 0) {
      setError("Select at least one model before sending a message.");
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

    const placeholderMessages: Message[] = targetModels.map((modelId, index) => ({
      id: `pending-${modelId}-${Date.now()}-${index}`,
      role: "assistant",
      content: "",
      model: modelId,
      chat_id: currentId,
      timestamp: new Date().toISOString(),
      message_type: "text",
      pending: true,
    }));

    const newMessages = [...updatedChat.messages, userMessage, ...placeholderMessages];
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
    setSelectedModels(targetModels);
    setModelStatuses(() => {
      const next: Record<string, "idle" | "pending"> = {};
      targetModels.forEach(id => {
        next[id] = "pending";
      });
      return next;
    });

    try {
      // Send message via WebSocket
      socket.sendMessage(currentId, trimmedPrompt, targetModels);

      // Note: Assistant response will come via WebSocket and be handled by the effect above

      const pendingChatId = currentId;
      // Fallback: Stop loading after 10 seconds in case WebSocket doesn't work
      setTimeout(() => {
        if (loading()) {
          console.log('⏰ Fallback: Stopping loading after timeout');
          setLoading(false);
          setModelStatuses(prev => {
            const entries = Object.keys(prev).map((key) => [key, "idle" as const]);
            return Object.fromEntries(entries);
          });
          setChats(prev =>
            prev.map(chat =>
              chat.id === pendingChatId
                ? {
                    ...chat,
                    messages: chat.messages?.filter(message => !message.pending) ?? [],
                  }
                : chat,
            ),
          );
        }
      }, 10000);

    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to send message");
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
      // Stop loading immediately when any message is received
      setLoading(false);

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
        model: message.model,
        timestamp: message.timestamp,
        message_type: message.message_type,
      };

      console.log("📝 New message to add:", newMessage);

      setChats(prev => {
        const updated = prev.map(chat => {
          if (chat.id !== currentId) {
            return chat;
          }

          const existing = [...(chat.messages ?? [])];
          if (newMessage.model) {
            const placeholderIndex = existing.findIndex(
              (msg) => msg.pending && msg.model === newMessage.model,
            );
            if (placeholderIndex >= 0) {
              existing.splice(placeholderIndex, 1);
            }
          }

          return {
            ...chat,
            messages: [...existing, newMessage],
          };
        });
        console.log("🔄 Updated chats:", updated);
        console.log("📊 Chat with new message:", updated.find(c => c.id === currentId)?.messages);
        return updated;
      });

      // Stop loading when any message is received
      setLoading(false);
      if (message.model) {
        setModelStatuses(prev => {
          if (!(message.model in prev) || prev[message.model] === "idle") {
            return prev;
          }
          return { ...prev, [message.model]: "idle" };
        });
      }
      } else {
        console.log("❌ Message chat ID does NOT match current chat ID:", {
          messageChatId: message.chat_id,
          currentId,
          chatIdsMatch: message.chat_id === currentId
        });
      }
    } else if (message.type === 'error') {
      const details = message.message || "An unexpected error occurred while processing the request.";
      console.error("🚨 Error received via WebSocket:", details);
      setError(details);
      setLoading(false);
      setModelStatuses(prev => {
        const next: Record<string, "idle" | "pending"> = {};
        Object.keys(prev).forEach(key => {
          next[key] = "idle";
        });
        return next;
      });
      setChats(prev =>
        prev.map(chat => ({
          ...chat,
          messages: chat.messages?.filter(m => !m.pending),
        })),
      );
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
      setSelectedModels([]);
      setModelStatuses({});
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
      const sidebarData: SidebarBootstrapData = await initializeFromAPI(token);
      const { folders: apiFolders, chats: apiChats } = sidebarData;

      const folderIdMap = new Map<number, string>();
      for (const folder of apiFolders) {
        folderIdMap.set(folder.id, folder.public_id);
      }

      const frontendChats: Chat[] = apiChats.map((apiChat: ApiChat) => {
        let messages: Message[] = [];
        try {
          const rawMessages = apiChat.messages ?? "[]";
          messages = JSON.parse(rawMessages) as Message[];
        } catch (e) {
          console.error("Failed to parse chat messages", e);
        }

        const folderPublicId =
          typeof apiChat.folder_id === "number" ? folderIdMap.get(apiChat.folder_id) : undefined;

        return {
          id: apiChat.public_id,
          public_id: apiChat.public_id,
          title: apiChat.title,
          messages,
          createdAt: new Date(apiChat.created_at),
          folderId: folderPublicId,
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
      const storedSession = loadStoredSession();
      if (storedSession) {
        setSession(storedSession);
      } else if (ENABLE_DEV_LOGIN) {
        // Optional development helper: auto-fetch dev token when explicitly enabled
        void fetchDevSession().then((devSession) => {
          if (devSession) {
            setSession(devSession);
            persistSession(devSession);
          }
        });
      }
    }
  });

  const sidebarActions: Actions = {
    ...actions,
    moveChat: async (chatId, target) => {
      const result = await actions.moveChat(chatId, target);
      if (result) {
        const normalizedFolderId =
          target.folderId && target.folderId !== "" ? target.folderId : undefined;
        setChats(prev =>
          prev.map(chat =>
            chat.id === chatId ? { ...chat, folderId: normalizedFolderId } : chat,
          ),
        );
      }
      return result;
    },
  };

  return (
    <ThemeProvider>
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
          onRenameChat={renameChat}
          onDeleteChat={deleteChat}
          onDeleteFolder={deleteFolder}
          actions={sidebarActions}
        />
            <MainArea
            prompt={prompt}
            setPrompt={setPrompt}
            attachedImages={attachedImages}
            setAttachedImages={setAttachedImages}
            selectedModels={selectedModels}
            setSelectedModels={setSelectedModels}
            models={models}
            modelStatuses={modelStatuses}
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
    </ThemeProvider>
  );
}
