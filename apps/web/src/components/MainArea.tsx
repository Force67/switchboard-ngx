import { Accessor, Setter, For } from "solid-js";
import TopRightControls from "./TopRightControls";
import Composer from "./Composer";

interface ModelOption {
  id: string;
  label: string;
  description?: string | null;
}

interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  reasoning?: string[];
}

interface Props {
  prompt: Accessor<string>;
  setPrompt: Setter<string>;
  selectedModel: Accessor<string>;
  setSelectedModel: Setter<string>;
  models: Accessor<ModelOption[]>;
  modelsLoading: Accessor<boolean>;
  modelsError: Accessor<string | null>;
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  currentMessages: Accessor<Message[]>;
  onSend: (event: Event) => void;
}

export default function MainArea(props: Props) {
  return (
    <div class="main">
      <TopRightControls />
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
        <For each={props.currentMessages()}>
          {(message) => (
            <div style={`padding: 20px; margin: 10px 20px; border-radius: 12px; background: ${message.role === 'user' ? 'var(--bg-3)' : 'var(--bg-2)'}; color: var(--text-0);`}>
              <div style="font-weight: bold; margin-bottom: 8px;">
                {message.role === 'user' ? 'You' : `Assistant${message.model ? ` (${message.model})` : ''}`}
              </div>
              <div style="white-space: pre-wrap;">{message.content}</div>
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
          )}
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
        selectedModel={props.selectedModel}
        setSelectedModel={props.setSelectedModel}
        models={props.models}
        modelsLoading={props.modelsLoading}
        modelsError={props.modelsError}
        loading={props.loading}
        onSend={props.onSend}
      />
    </div>
  );
}