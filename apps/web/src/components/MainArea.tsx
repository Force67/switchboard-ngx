import { Accessor, Setter, For, createMemo } from "solid-js";
import TopRightControls from "./TopRightControls";
import Composer from "./Composer";
import ModelPickerPanel from "./model-picker/ModelPickerPanel";
import { ModelMeta } from "./model-picker/models";
import LatexRenderer from "./LatexRenderer";

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
  selectedModel: Accessor<string>;
  setSelectedModel: Setter<string>;
  models: Accessor<ModelOption[]>;
  modelsLoading: Accessor<boolean>;
  modelsError: Accessor<string | null>;
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  modelPickerOpen: Accessor<boolean>;
  setModelPickerOpen: Setter<boolean>;
  currentMessages: Accessor<Message[]>;
  session: Accessor<SessionData | null>;
  onSend: (event: Event) => void;
  onLogout: () => void;
}

export default function MainArea(props: Props) {
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

  return (
    <div class="main">
      <TopRightControls session={props.session} onLogout={props.onLogout} />
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
     </div>
   );
 }