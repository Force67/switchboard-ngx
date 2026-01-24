import { Accessor, Setter, For, createMemo, createSignal, Show } from "solid-js";
import { onMount, onCleanup } from "solid-js";
import Composer from "./Composer";
import ModelPickerPanel from "./model-picker/ModelPickerPanel";
import { ModelMeta } from "./model-picker/models";
import MarkdownRenderer from "./MarkdownRenderer";
import GroupChatManager from "./GroupChatManager";
import ChatPropertiesSidebar, { ChatProperties } from "./ChatPropertiesSidebar";
import type { Chat, Message } from "../types/chat";
import "./chat-properties.css";

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

interface SessionData {
  token: string;
  user: {
    id: string;
    email?: string | null;
    display_name?: string | null;
  };
  expires_at: string;
}

interface Props {
  prompt: Accessor<string>;
  setPrompt: Setter<string>;
  attachedImages: Accessor<File[]>;
  setAttachedImages: Setter<File[]>;
  selectedModels: Accessor<string[]>;
  setSelectedModels: Setter<string[]>;
  models: Accessor<ModelOption[]>;
  modelStatuses: Accessor<Record<string, "idle" | "pending">>;
  modelsLoading: Accessor<boolean>;
  modelsError: Accessor<string | null>;
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  authError?: Accessor<string | null>;
  modelPickerOpen: Accessor<boolean>;
  setModelPickerOpen: Setter<boolean>;
  currentMessages: Accessor<Message[]>;
  currentChat?: Accessor<Chat | null>;
  session: Accessor<SessionData | null>;
  onSend: (event: Event) => void;
  onOpenSidebar?: () => void;
  chatProperties?: Accessor<ChatProperties>;
  setChatProperties?: Setter<ChatProperties>;
  propertiesSidebarOpen?: Accessor<boolean>;
  setPropertiesSidebarOpen?: Setter<boolean>;
}

export default function MainArea(props: Props) {
  const [showGroupManager, setShowGroupManager] = createSignal(false);
  const sessionUserId = createMemo(() => props.session()?.user.id);

  // Local chat properties state if not provided via props
  const [localChatProperties, setLocalChatProperties] = createSignal<ChatProperties>({
    webSearchEnabled: false,
    temperature: 0.7,
    maxTokens: 4096,
    systemPrompt: ""
  });

  const chatProperties = () => props.chatProperties?.() ?? localChatProperties();
  const setChatProperties = props.setChatProperties ?? setLocalChatProperties;

  // Use props for sidebar state, fallback to local state
  const [localSidebarOpen, setLocalSidebarOpen] = createSignal(false);
  const propertiesSidebarOpen = () => props.propertiesSidebarOpen?.() ?? localSidebarOpen();
  const setPropertiesSidebarOpen = props.setPropertiesSidebarOpen ?? setLocalSidebarOpen;

  const convertedModels = createMemo((): ModelMeta[] => {
    return props.models().map(model => {
      // Extract provider from model ID (e.g., "openai/gpt-4o" → "openai")
      const provider = model.id.includes('/') ? model.id.split('/')[0] : undefined;

      // Build badges array from all capability flags
      const badges: ModelMeta['badges'] = [];
      if (model.supports_vision || model.supports_images) badges.push('vision');
      if (model.supports_tools || model.supports_tool_use || model.supports_function_calling) badges.push('tools');
      if (model.supports_agents) badges.push('agent');
      if (model.supports_reasoning) badges.push('reasoning');

      return {
        id: model.id,
        name: model.label,
        description: model.description ?? undefined,
        badges,
        tier: undefined,
        disabled: false,
        group: undefined,
        provider,
        pricing: model.pricing,
      };
    });
  });

  const modelSelectorLabel = createMemo(() => {
    const ids = props.selectedModels();
    const available = props.models();
    const names = ids
      .map(id => available.find(model => model.id === id)?.label || id)
      .filter(Boolean);

    if (names.length === 0) {
      return "Select Models";
    }
    if (names.length === 1) {
      return names[0];
    }
    if (names.length === 2) {
      return `${names[0]}, ${names[1]}`;
    }
    return `${names[0]} + ${names.length - 1}`;
  });

  const pendingModels = createMemo(() => {
    const statuses = props.modelStatuses();
    const available = props.models();
    return props.selectedModels()
      .map((id) => ({
        id,
        label: available.find(model => model.id === id)?.label || id,
        status: statuses[id] ?? "idle",
      }))
      .filter(entry => entry.status === "pending");
  });

  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.shiftKey && e.key === 'P') {
        e.preventDefault();
        props.setModelPickerOpen(true);
      }
      if (e.shiftKey && e.key === 'S') {
        e.preventDefault();
        setPropertiesSidebarOpen(prev => !prev);
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    onCleanup(() => {
      document.removeEventListener('keydown', handleKeyDown);
    });
  });

  return (
    <div class="main">
      {props.currentChat?.()?.isGroup && (
        <div style={{
          padding: "8px 20px",
          background: "var(--bg-2)",
          borderBottom: "1px solid rgba(255,255,255,0.05)"
        }}>
          <button
            onClick={() => setShowGroupManager(true)}
            style={{
              padding: "6px 12px",
              borderRadius: "6px",
              border: "1px solid rgba(255,255,255,0.2)",
              background: "var(--bg-3)",
              color: "var(--text-0)",
              cursor: "pointer",
              fontSize: "12px"
            }}
          >
            👥 Manage Group
          </button>
        </div>
      )}
      <div class="content-well">
        {props.error() && (
          <div style="padding: 20px; color: #ff6b6b; background: rgba(255,107,107,0.1); border-radius: 8px; margin: 20px;">
            {props.error()}
          </div>
        )}
        {props.modelsError() && (
          <div style="padding: 20px; color: #ff6b6b; background: rgba(255,107,107,0.1); border-radius: 8px; margin: 20px;">
            Models error: {props.modelsError()}
          </div>
        )}
        {props.authError?.() && (
          <div style="padding: 20px; color: #ff6b6b; background: rgba(255,107,107,0.1); border-radius: 8px; margin: 20px;">
            Authentication error: {props.authError!()}
          </div>
        )}
        <Show when={pendingModels().length > 0}>
          <div class="model-pending-banner">
            <span class="banner-title">Models still thinking</span>
            <div class="banner-models">
              <For each={pendingModels()}>
                {(item) => (
                  <span class="pending-chip" title={`${item.label} is working...`}>
                    <span class="pending-icon">🕺</span>
                    <span class="pending-label">{item.label}</span>
                  </span>
                )}
              </For>
            </div>
          </div>
        </Show>
                  <For each={props.currentMessages()}>
          {(message, i) => {
            const isCurrentUser = () => {
              const sessionId = sessionUserId();
              if (!sessionId || message.user_id == null) return false;
              return String(message.user_id) === String(sessionId);
            };
            const modelInfo = () => props.models().find(m => m.id === message.model);
            const modelLabel = modelInfo()?.label || message.model || "Assistant";
            const displayName = message.role === 'user'
              ? (isCurrentUser() ? 'You' : `User ${message.user_id ?? "?"}`)
              : `Assistant (${modelLabel})`;
            const isPendingMessage = message.pending === true;

            // Only animate while this user message has no assistant message after it
            const shouldAnimate = () => {
              if (message.role !== "user") return false;
              const arr = props.currentMessages();        // always read the live array
              for (let j = i() + 1; j < arr.length; j++) {
                if (arr[j].role === "assistant") return false; // assistant reply arrived
              }
              return true; // still waiting on assistant reply after this user msg
            };

            return (
              <>
                <div
                  style={`padding: 20px; margin: 10px 20px; border-radius: 12px; background: ${message.role === 'user' ? 'var(--bg-3)' : '#1a2332'}; color: var(--text-0); position: relative; ${shouldAnimate() ? 'border: 3px solid yellow;' : ''}`}
                  classList={{
                    'user-message-blowing': shouldAnimate()
                  }}
                >
                  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <div style="font-weight: bold;">
                      {displayName}
                    </div>
                    {message.timestamp && (
                      <small style="color: var(--text-1);">
                        {new Date(message.timestamp).toLocaleTimeString()}
                      </small>
                    )}
                  </div>
                  {isPendingMessage ? (
                    <div class="assistant-pending-message" title={`${modelLabel} is thinking...`}>
                      <span class="pending-icon">🕺</span>
                      <span class="pending-label">{modelLabel} is thinking...</span>
                    </div>
                  ) : (
                    <MarkdownRenderer content={message.content} />
                  )}
                  {message.usage && (
                    <small style="color: var(--text-1); margin-top: 8px; display: block;">
                      Tokens: {message.usage.prompt_tokens} prompt, {message.usage.completion_tokens} completion
                    </small>
                  )}
                  {message.reasoning && message.reasoning.length > 0 && (
                    <details style="margin-top: 8px;">
                      <summary style="cursor: pointer; color: var(--text-1);">Reasoning</summary>
                      <ol style="margin-top: 4px;">
                        <For each={message.reasoning}>
                          {(step) => <li style="color: var(--text-1);">{step}</li>}
                        </For>
                      </ol>
                    </details>
                  )}
                </div>
                <Show when={shouldAnimate()}>
                  <div class="blowing-particles">
                    <div class="particle particle-1">✨</div>
                    <div class="particle particle-2">🌟</div>
                    <div class="particle particle-3">💫</div>
                    <div class="particle particle-4">⭐</div>
                    <div class="particle particle-5">✨</div>
                  </div>
                </Show>
              </>
            );
          }}
        </For>
        {props.loading() && (
          <div style="padding: 20px; margin: 10px 20px; color: var(--text-1);">
            Assistant is thinking...
          </div>
        )}
      </div>
        <Composer
          prompt={props.prompt}
          setPrompt={props.setPrompt}
          attachedImages={props.attachedImages}
          setAttachedImages={props.setAttachedImages}
          selectedModelIds={props.selectedModels}
          modelStatuses={props.modelStatuses}
          models={props.models}
          modelsLoading={props.modelsLoading}
          modelsError={props.modelsError}
          loading={props.loading}
          onSend={props.onSend}
          onOpenModelPicker={() => props.setModelPickerOpen(true)}
          onMentionSelect={(modelId) => {
            props.setSelectedModels(prev => {
              if (prev.includes(modelId)) return prev;
              return [...prev, modelId];
            });
          }}
          modelSelectorLabel={modelSelectorLabel}
        />
        {props.modelPickerOpen() && (
          <div
            style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 49;"
            onClick={() => props.setModelPickerOpen(false)}
          >
            <ModelPickerPanel
              models={convertedModels()}
              selectedIds={props.selectedModels()}
              onToggle={(id) => {
                props.setSelectedModels(prev => {
                  if (prev.includes(id)) {
                    return prev.filter(existing => existing !== id);
                  }
                  return [...prev, id];
                });
              }}
              multiSelect
              autoFocusSearch
            />
          </div>
        )}

        {showGroupManager() && props.currentChat?.() && props.session() && (
          <GroupChatManager
            chatId={props.currentChat()!.id}
            session={props.session()!}
            onClose={() => setShowGroupManager(false)}
          />
        )}

        {/* Chat Properties Sidebar */}
        <ChatPropertiesSidebar
          isOpen={propertiesSidebarOpen}
          onClose={() => setPropertiesSidebarOpen(false)}
          properties={chatProperties}
          setProperties={setChatProperties}
          currentModelId={createMemo(() => props.selectedModels()[0] ?? null)}
        />
      </div>
  );
}
