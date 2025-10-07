import { Accessor, Setter, For, createMemo, createSignal, Show } from "solid-js";
import { onMount, onCleanup } from "solid-js";
import TopRightControls from "./TopRightControls";
import Composer from "./Composer";
import ModelPickerPanel from "./model-picker/ModelPickerPanel";
import { ModelMeta } from "./model-picker/models";
import LatexRenderer from "./LatexRenderer";
import GroupChatManager from "./GroupChatManager";

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

interface Message {
  id?: string;
  chat_id?: string;
  user_id?: number;
  role: "user" | "assistant" | "system";
  content: string;
  model?: string;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  reasoning?: string[];
  timestamp?: string;
  message_type?: string;
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

interface Chat {
  id: string;
  title: string;
  isGroup?: boolean;
}

interface Props {
  prompt: Accessor<string>;
  setPrompt: Setter<string>;
  attachedImages: Accessor<File[]>;
  setAttachedImages: Setter<File[]>;
  selectedModel: Accessor<string>;
  setSelectedModel: Setter<string>;
  models: Accessor<ModelOption[]>;
  connectionStatus?: Accessor<{ status: string; error?: string }>;
  modelsLoading: Accessor<boolean>;
  modelsError: Accessor<string | null>;
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  modelPickerOpen: Accessor<boolean>;
  setModelPickerOpen: Setter<boolean>;
  currentMessages: Accessor<Message[]>;
  currentChat?: Accessor<Chat | null>;
  session: Accessor<SessionData | null>;
  onSend: (event: Event) => void;
  onLogout: () => void;
}

export default function MainArea(props: Props) {
  const [showGroupManager, setShowGroupManager] = createSignal(false);

  const convertedModels = createMemo((): ModelMeta[] => {
    return props.models().map(model => ({
      id: model.id,
      name: model.label,
      badges: [
        ...(model.supports_reasoning ? ['reasoning' as const] : []),
        ...(model.supports_images ? ['vision' as const] : []),
      ],
      tier: undefined,
      disabled: false,
      group: undefined,
      pricing: model.pricing,
    }));
  });

  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.shiftKey && e.key === 'P') {
        e.preventDefault();
        props.setModelPickerOpen(true);
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    onCleanup(() => {
      document.removeEventListener('keydown', handleKeyDown);
    });
  });

  return (
    <div class="main">
      <TopRightControls session={props.session} onLogout={props.onLogout} />
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
        {props.connectionStatus && (() => {
          const status = props.connectionStatus!().status;
          const error = props.connectionStatus!().error;
          if (status === 'connected') return null;
          return (
            <div style={`padding: 10px 20px; border-radius: 8px; margin: 20px; background: ${status === 'error' ? 'rgba(255,107,107,0.1)' : 'rgba(255,193,7,0.1)'}; color: ${status === 'error' ? '#ff6b6b' : '#ffc107'};`}>
              {status === 'error' ? `Connection Error: ${error || 'Unknown error'}` :
               status === 'connecting' ? 'Connecting...' :
               status === 'disconnected' ? 'Disconnected - messages may not be delivered' :
               `Connection: ${status}`}
            </div>
          );
        })()}
        <For each={props.currentMessages()}>
          {(message) => {
            const isCurrentUser = message.user_id === 1; // Assuming user_id 1 is current user
            const displayName = message.role === 'user'
              ? (isCurrentUser ? 'You' : `User ${message.user_id}`)
              : `Assistant${message.model ? ` (${message.model})` : ''}`;

            return (
              <div style={`padding: 20px; margin: 10px 20px; border-radius: 12px; background: ${message.role === 'user' ? 'var(--bg-3)' : 'var(--bg-2)'}; color: var(--text-0);`}>
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
                <LatexRenderer content={message.content} />
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
          selectedModel={props.selectedModel}
          setSelectedModel={props.setSelectedModel}
          models={props.models}
          modelsLoading={props.modelsLoading}
          modelsError={props.modelsError}
          loading={props.loading}
          onSend={props.onSend}
          onOpenModelPicker={() => props.setModelPickerOpen(true)}
        />
        {props.modelPickerOpen() && (
          <div
            style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 49;"
            onClick={() => props.setModelPickerOpen(false)}
          >
            <ModelPickerPanel
              models={convertedModels()}
              selectedId={props.selectedModel()}
              onSelect={(id) => {
                props.setSelectedModel(id);
                props.setModelPickerOpen(false);
              }}
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
      </div>
  );
}