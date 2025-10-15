import { createSignal, createEffect, Accessor, Setter, For, Show } from "solid-js";
import ModelSelector from "./ModelSelector";
import Chip from "./Chip";
import SendButton from "./SendButton";
import ModelPickerPanel from "./model-picker/ModelPickerPanel";
import ColoredTextarea from "./ColoredTextarea";
import "./model-picker/model-picker.css";

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

interface Props {
  prompt: Accessor<string>;
  setPrompt: Setter<string>;
  attachedImages: Accessor<File[]>;
  setAttachedImages: Setter<File[]>;
  selectedModelIds: Accessor<string[]>;
  models: Accessor<ModelOption[]>;
  modelsLoading: Accessor<boolean>;
  modelsError: Accessor<string | null>;
  loading: Accessor<boolean>;
  onSend: (event: Event) => void;
  onOpenModelPicker: () => void;
  onMentionSelect?: (modelId: string) => void;
  modelStatuses: Accessor<Record<string, "idle" | "pending">>;
  modelSelectorLabel: Accessor<string>;
}

export default function Composer(props: Props) {
  let textareaRef: HTMLTextAreaElement | undefined;
  const [showModelPicker, setShowModelPicker] = createSignal(false);
  const [cursorPosition, setCursorPosition] = createSignal(0);

  const handleSend = (event: Event) => {
    props.onSend(event);
  };

  createEffect(() => {
    if (textareaRef) {
      textareaRef.style.height = "auto";
      textareaRef.style.height = `${Math.min(textareaRef.scrollHeight, 120)}px`;
    }
  });

  // Note: Click outside is now handled by the backdrop overlay

  const checkForAtSymbol = (text: string, position: number) => {
    if (position > 0 && text[position - 1] === '@') {
      const wordStart = text.lastIndexOf(' ', position - 1) + 1;
      const beforeAt = text.substring(wordStart, position - 1);
      if (beforeAt === '') {
        setShowModelPicker(true);
        setCursorPosition(position);
        return true;
      }
    }
    setShowModelPicker(false);
    return false;
  };

  const handleInputChange = (e: InputEvent) => {
    const textarea = e.currentTarget as HTMLTextAreaElement;
    const newValue = textarea.value;
    const newPosition = textarea.selectionStart;

    props.setPrompt(newValue);

    checkForAtSymbol(newValue, newPosition);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault();
      props.onSend(e);
    } else if (e.key === 'Escape' && showModelPicker()) {
      e.preventDefault();
      setShowModelPicker(false);
    }
  };

  const handleModelSelect = (modelId: string) => {
    const text = props.prompt();
    const cursorPos = cursorPosition();
    const model = props.models().find(m => m.id === modelId);

    if (model) {
      const beforeAt = text.substring(0, cursorPos - 1);
      const afterCursor = text.substring(cursorPos);
      const normalized = model.label.toLowerCase().replace(/\s+/g, '');
      const hasTrailingWhitespace = beforeAt.length === 0 || /\s$/.test(beforeAt);
      const prefix = hasTrailingWhitespace ? beforeAt : `${beforeAt} `;
      const newText = prefix + '@' + normalized + ' ' + afterCursor;
      props.setPrompt(newText);
      props.onMentionSelect?.(modelId);
    }

    setShowModelPicker(false);

    setTimeout(() => {
      if (textareaRef) {
        const model = props.models().find(m => m.id === modelId);
        if (model) {
          const modelName = model.label.toLowerCase().replace(/\s+/g, '');
          const newCursorPos = cursorPos + modelName.length;
          textareaRef.focus();
          textareaRef.setSelectionRange(newCursorPos, newCursorPos);
        }
      }
    }, 0);
  };

  return (
    <>
      <form class="composer" onSubmit={handleSend}>
      <div class="ta" style="position: relative;">
        <ColoredTextarea
          value={props.prompt}
          onInput={props.setPrompt}
          onKeyDown={handleKeyDown}
          onInputCustom={handleInputChange}
          placeholder="Type your message..."
          style={
            "width: 100%; min-height: 44px; max-height: 120px; border: none; outline: none; font-size: 14px; font-family: inherit; line-height: 1.5;"
          }
          ref={(el) => textareaRef = el}
        />
        <Show when={showModelPicker()}>
          <>
            <div
              style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.1); z-index: 999;"
              onClick={() => setShowModelPicker(false)}
            />
            <div
              style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; display: flex; align-items: center; justify-content: center; z-index: 1000;"
              onClick={(e) => e.stopPropagation()}
            >
              <div style="transform: translateY(-700px);">
                <ModelPickerPanel
                  models={props.models().map(model => ({
                    id: model.id,
                    name: model.label,
                    badges: [
                      ...(model.supports_reasoning ? ["reasoning" as const] : []),
                      ...(model.supports_images ? ["vision" as const] : []),
                    ],
                    tier: undefined,
                    disabled: false,
                    group: undefined,
                    pricing: model.pricing,
                  }))}
                  selectedIds={props.selectedModelIds()}
                  onToggle={handleModelSelect}
                  autoFocusSearch
                />
              </div>
            </div>
          </>
        </Show>
      </div>
      {props.attachedImages().length > 0 && (
        <div class="previews">
          <For each={props.attachedImages()}>
            {(file, index) => (
              <div class="preview">
                <img src={URL.createObjectURL(file)} alt={file.name} />
                <button onClick={() => props.setAttachedImages(prev => prev.filter((_, i) => i !== index()))}>×</button>
              </div>
            )}
          </For>
        </div>
      )}
      <div class="foot">
        <ModelSelector
          label={props.modelSelectorLabel()}
          onOpen={props.onOpenModelPicker}
          loading={props.modelsLoading()}
        />
        <div class="model-status-tray">
          <For each={props.selectedModelIds()}>
            {(id) => {
              const model = props.models().find(m => m.id === id);
              const status = props.modelStatuses()[id] ?? "idle";
              const icon = status === "pending" ? "🕺" : "✨";
              const label = model?.label ?? id;
              const title =
                status === "pending"
                  ? `${label} is still thinking...`
                  : `${label} is ready`;
              return (
                <span class="model-status-chip" data-status={status} title={title}>
                  <span class="model-status-icon">{icon}</span>
                  <span class="model-status-label">{label}</span>
                </span>
              );
            }}
          </For>
        </div>
        <Chip label="Search" icon={<svg viewBox="0 0 12 12"><circle cx="4.5" cy="4.5" r="3.5" stroke-width="1.5" fill="none" /><path d="M7.5 7.5L10.5 10.5" stroke-width="1.5" stroke-linecap="round" /></svg>} />
        <Chip circular icon={<svg viewBox="0 0 12 12"><path d="M3 4.5a.5.5 0 0 0-.5.5v3a.5.5 0 0 0 .5.5h6a.5.5 0 0 0 .5-.5V5a.5.5 0 0 0-.5-.5h-2a.5.5 0 0 1 0-1h2A1.5 1.5 0 0 1 10.5 5v3a1.5 1.5 0 0 1-1.5 1.5h-6A1.5 1.5 0 0 1 1.5 8V5A1.5 1.5 0 0 1 3 3.5h2a.5.5 0 0 0 0-1h-2z" /></svg>} onClick={() => {
          const input = document.createElement('input');
          input.type = 'file';
          input.accept = 'image/*';
          input.multiple = true;
          input.onchange = (e) => {
            const files = Array.from((e.target as HTMLInputElement).files || []);
            props.setAttachedImages(prev => [...prev, ...files]);
          };
          input.click();
        }} />
        <SendButton disabled={!props.prompt().trim() || props.loading()} type="submit" />
      </div>
    </form>

      </>
  );
}
