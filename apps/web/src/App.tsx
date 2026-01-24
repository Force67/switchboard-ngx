import { createSignal, createEffect, onMount, createMemo } from "solid-js";
import "./theme.css";
import "./app.css";
import { ThemeProvider } from "./contexts/ThemeContext";
import Sidebar from "./components/Sidebar";
import MainArea from "./components/MainArea";
import TopRightControls from "./components/TopRightControls";
import type { ChatProperties } from "./components/ChatPropertiesSidebar";
import { apiService } from "./api";
import type { ApiChat } from "./api";
import { API_BASE } from "./config";
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

const DEFAULT_MODEL = import.meta.env.VITE_DEFAULT_MODEL ?? "";
const GITHUB_REDIRECT_PATH =
  import.meta.env.VITE_GITHUB_REDIRECT_PATH ?? "/auth/callback";
const SESSION_KEY = "switchboard.session";
const AUTO_DEV_SESSION_ENABLED =
  import.meta.env.VITE_ENABLE_DEV_LOGIN === "true" ||
  (typeof window !== "undefined" &&
    (window.location.hostname === "localhost" ||
      window.location.hostname === "127.0.0.1"));

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
  supports_tools?: boolean;
  supports_agents?: boolean;
  supports_function_calling?: boolean;
  supports_vision?: boolean;
  supports_tool_use?: boolean;
  supports_structured_outputs?: boolean;
  supports_streaming?: boolean;
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
    const response = await fetch(`${API_BASE}/api/v1/auth/dev/token`);
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
  const [sidebarOpen, setSidebarOpen] = createSignal(false);
  const [chatProperties, setChatProperties] = createSignal<ChatProperties>({
    webSearchEnabled: false,
    temperature: 0.7,
    maxTokens: 4096,
    systemPrompt: ""
  });
  const [propertiesSidebarOpen, setPropertiesSidebarOpen] = createSignal(false);

  // Check if any properties are active (for indicator dot)
  const hasActiveProperties = createMemo(() => {
    const p = chatProperties();
    return p.webSearchEnabled || (p.systemPrompt !== undefined && p.systemPrompt.length > 0);
  });

  // WebSocket integration
  const socket = useSocket(() => session()?.token || null);
  let devSessionBootstrapInFlight = false;

  const redirectUri = () => `${window.location.origin}${GITHUB_REDIRECT_PATH}`;

  const persistSession = (
    value: SessionData | null,
    options: { suppressAutoBootstrap?: boolean } = {},
  ) => {
    setSession(value);
    if (typeof window === "undefined") {
      return;
    }
    if (value) {
      window.localStorage.setItem(SESSION_KEY, JSON.stringify(value));
    } else {
      window.localStorage.removeItem(SESSION_KEY);
      if (AUTO_DEV_SESSION_ENABLED && !options.suppressAutoBootstrap) {
        void bootstrapDevSession();
      }
    }
  };

  async function bootstrapDevSession() {
    if (
      devSessionBootstrapInFlight ||
      session() ||
      !AUTO_DEV_SESSION_ENABLED
    ) {
      return;
    }

    devSessionBootstrapInFlight = true;
    try {
      const devSession = await fetchDevSession();
      if (devSession) {
        persistSession(devSession, { suppressAutoBootstrap: true });
      }
    } catch (error) {
      console.error("Failed to bootstrap development session:", error);
    } finally {
      devSessionBootstrapInFlight = false;
    }
  }

  const finalizeGithubLogin = async (code: string, state: string) => {
    setAuthError(null);
    try {
      const response = await fetch(`${API_BASE}/api/v1/auth/github/callback`, {
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
        `${API_BASE}/api/v1/auth/github/login?redirect_uri=${encodeURIComponent(redirectUri())}`,
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
      const response = await fetch(`${API_BASE}/api/v1/models`, {
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
    persistSession(null, { suppressAutoBootstrap: true });
    setAuthError(null);
    setAuthenticating(false);
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
        if (availableModels.some(model => model.id === "debug/echo")) {
          return "debug/echo";
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

    const pendingChatId = currentId;

    try {
      const buildChatUrls = () => {
        const urls = new Set<string>();
        urls.add(`${API_BASE}/api/v1/chat`);
        if (typeof window !== "undefined") {
          urls.add("/api/chat"); // dev proxy fallback
          urls.add(`${window.location.origin}/api/chat`);
        }
        return Array.from(urls);
      };

      const sendChatRequest = async (formData: FormData) => {
        const urls = buildChatUrls();
        let lastError: unknown = null;

        for (const url of urls) {
          try {
            const response = await fetch(url, {
              method: "POST",
              headers: {
                Authorization: `Bearer ${activeSession.token}`,
              },
              body: formData,
            });

            if (!response.ok) {
              const text = await response.text();
              throw new Error(text || response.statusText);
            }

            return (await response.json()) as ChatResponse;
          } catch (err) {
            lastError = err;
            // Retry on network errors (TypeError) with the next URL fallback
            if (err instanceof TypeError) {
              continue;
            }
            throw err;
          }
        }

        throw lastError ?? new Error("Failed to reach chat API");
      };

      const results = await Promise.allSettled(
        targetModels.map(async (modelId) => {
          const formData = new FormData();
          formData.append("prompt", trimmedPrompt);
          formData.append("model", modelId);

          // Include chat properties in the request
          const props = chatProperties();
          if (props.webSearchEnabled) {
            formData.append("web_search", "true");
          }
          if (props.temperature !== undefined) {
            formData.append("temperature", props.temperature.toString());
          }
          if (props.maxTokens !== undefined) {
            formData.append("max_tokens", props.maxTokens.toString());
          }
          if (props.systemPrompt) {
            formData.append("system_prompt", props.systemPrompt);
          }

          const data = await sendChatRequest(formData);

          const assistantMessage: Message = {
            id: `assistant-${modelId}-${Date.now()}`,
            chat_id: pendingChatId,
            role: "assistant",
            content: data.content,
            model: data.model,
            timestamp: new Date().toISOString(),
            usage: data.usage,
            reasoning: data.reasoning,
            web_search_sources: data.web_search_sources,
          };

          setChats(prev =>
            prev.map(chat => {
              if (chat.id !== pendingChatId) return chat;
              const existing = [...(chat.messages ?? [])];
              const placeholderIndex = existing.findIndex(
                (msg) => msg.pending && msg.model === modelId,
              );
              if (placeholderIndex >= 0) {
                existing.splice(placeholderIndex, 1);
              }
              return { ...chat, messages: [...existing, assistantMessage] };
            }),
          );

          setModelStatuses(prev => ({ ...prev, [modelId]: "idle" }));
        }),
      );

      const failed = results.find((result) => result.status === "rejected") as PromiseRejectedResult | undefined;
      if (failed) {
        throw failed.reason;
      }
    } catch (err) {
      const friendlyMessage =
        err instanceof TypeError
          ? `Network error reaching chat API. Tried ${API_BASE} and local proxies. Ensure the backend is running and VITE_API_BASE matches it.`
          : err instanceof Error
            ? err.message
            : "Failed to send message";

      setError(friendlyMessage);
      setModelStatuses(prev => {
        const next: Record<string, "idle" | "pending"> = {};
        Object.keys(prev).forEach(key => {
          next[key] = "idle";
        });
        return next;
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
    } finally {
      setLoading(false);
    }
  };

  const normalizeSocketMessage = (raw: any) => {
    if (!raw || raw.type !== "message") return null;

    const payload = raw.message ?? raw;
    const chatId = raw.chat_id ?? payload.chat_id;
    if (!chatId) return null;

    const rawUserId = payload.user_id ?? payload.sender_id ?? raw.user_id;
    const sessionUserId = session()?.user.id;
    const role: Message["role"] =
      payload.role ??
      (rawUserId && sessionUserId && String(rawUserId) === String(sessionUserId)
        ? "user"
        : "assistant");

    return {
      chatId,
      message: {
        id: payload.id ?? raw.message_id ?? payload.message_id,
        chat_id: chatId,
        user_id: typeof rawUserId === "string" ? Number(rawUserId) || undefined : rawUserId,
        role,
        content: payload.content ?? raw.content ?? "",
        model: payload.model ?? raw.model,
        timestamp: payload.timestamp ?? payload.created_at ?? raw.timestamp ?? new Date().toISOString(),
        message_type: payload.message_type ?? raw.message_type,
      } as Message,
    };
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
      const normalized = normalizeSocketMessage(message);
      if (!normalized) {
        console.log("❌ Unable to normalize message payload", message);
        return;
      }

      const { chatId, message: incoming } = normalized;

      if (chatId !== currentId) {
        console.log("❌ Message chat ID does NOT match current chat ID:", {
          messageChatId: chatId,
          currentId,
          chatIdsMatch: chatId === currentId
        });
        return;
      }

      // Stop loading immediately when any message is received
      setLoading(false);

      console.log("✅ Message chat ID matches current chat ID - processing message");
      // Check if this message already exists in the chat (user messages are added immediately)
      const currentChat = chats().find(c => c.id === currentId);
      console.log("🔍 Current chat found:", !!currentChat);
      console.log("📊 Current chat messages count:", currentChat?.messages?.length || 0);

      const messageExists = currentChat?.messages?.some(m => m.id && incoming.id && m.id === incoming.id);
      console.log("🔍 Message exists in chat:", messageExists);
      console.log("🔍 Looking for message ID:", incoming.id);
      console.log("🔍 Current chat message IDs:", currentChat?.messages?.map(m => m.id));

      if (messageExists) {
        // This is a user message that was already added to UI, skip
        console.log("⏭️ Message already exists, skipping (user message echo)");
        return;
      }

      // Check if this looks like a user message by comparing with the last user message
      const lastUserMessage = currentChat?.messages
        ?.filter(m => m.role === 'user')
        .pop();

      if (incoming.role === 'user' && lastUserMessage && lastUserMessage.content === incoming.content) {
        console.log("⏭️ Skipping user message echo (content matches last user message)");
        return;
      }

      console.log("🤖 New message detected, adding to chat...");
      // All messages received via WebSocket that aren't already in the chat should be assistant responses
      // User messages are added immediately to UI when sent, so WebSocket messages are always assistant responses

      setChats(prev => {
        const updated = prev.map(chat => {
          if (chat.id !== currentId) {
            return chat;
          }

          const existing = [...(chat.messages ?? [])];
          if (incoming.role === "assistant") {
            const placeholderIndex = existing.findIndex(
              (msg) => msg.pending && (incoming.model ? msg.model === incoming.model : true),
            );
            if (placeholderIndex >= 0) {
              existing.splice(placeholderIndex, 1);
            }
          }

          return {
            ...chat,
            messages: [...existing, incoming],
          };
        });
        console.log("🔄 Updated chats:", updated);
        console.log("📊 Chat with new message:", updated.find(c => c.id === currentId)?.messages);
        return updated;
      });

      // Stop loading when any message is received
      setLoading(false);
      setModelStatuses(prev => {
        if (incoming.model) {
          if (!(incoming.model in prev) || prev[incoming.model] === "idle") {
            return prev;
          }
          return { ...prev, [incoming.model]: "idle" };
        }

        if (Object.keys(prev).length === 0) {
          return prev;
        }

        const reset: Record<string, "idle"> = {};
        Object.keys(prev).forEach(key => {
          reset[key] = "idle";
        });
        return reset;
      });
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
      } else if (AUTO_DEV_SESSION_ENABLED) {
        void bootstrapDevSession();
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
      <div class="app-container">
        <TopRightControls
          session={session}
          onLogout={logout}
          connectionStatus={createMemo(() => {
            const state = socket.state();
            return {
              status: state.status,
              error: state.error || undefined
            };
          })}
          propertiesSidebarOpen={propertiesSidebarOpen}
          onToggleProperties={() => setPropertiesSidebarOpen(prev => !prev)}
          hasActiveProperties={hasActiveProperties}
          onOpenSidebar={() => setSidebarOpen(true)}
        />
        <div class="app-body">
          <Sidebar
            session={session}
            chats={chats}
            currentChatId={currentChatId}
            onLogin={beginGithubLogin}
            onLogout={logout}
            onNewChat={newChat}
            onNewGroupChat={newGroupChat}
            onSelectChat={(chatId) => {
              selectChat(chatId);
              setSidebarOpen(false); // Close sidebar on mobile after selecting chat
            }}
            onRenameChat={renameChat}
            onDeleteChat={deleteChat}
            onDeleteFolder={deleteFolder}
            actions={sidebarActions}
            isOpen={sidebarOpen}
            onClose={() => setSidebarOpen(false)}
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
            authError={authError}
            modelPickerOpen={modelPickerOpen}
            setModelPickerOpen={setModelPickerOpen}
            session={session}
            currentMessages={createMemo(() => {
              const currentId = currentChatId();
              const currentChat = chats().find(c => c.id === currentId);
              const messages = currentChat ? currentChat.messages : [];
              return messages;
            })}
            currentChat={createMemo(() => {
              const currentId = currentChatId();
              return chats().find(c => c.id === currentId) || null;
            })}
            onSend={handleSubmit}
            onOpenSidebar={() => setSidebarOpen(true)}
            chatProperties={chatProperties}
            setChatProperties={setChatProperties}
            propertiesSidebarOpen={propertiesSidebarOpen}
            setPropertiesSidebarOpen={setPropertiesSidebarOpen}
          />
        </div>
      </div>
    </ThemeProvider>
  );
}
