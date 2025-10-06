import { createSignal, createEffect, Accessor, Setter } from "solid-js";
import ModelSelector from "./ModelSelector";
import Chip from "./Chip";
import SendButton from "./SendButton";

interface ModelOption {
  id: string;
  label: string;
  description?: string | null;
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
  onSend: (event: Event) => void;
}

export default function Composer(props: Props) {
  let textareaRef: HTMLTextAreaElement | undefined;

  const handleSend = (event: Event) => {
    props.onSend(event);
  };

  createEffect(() => {
    if (textareaRef) {
      textareaRef.style.height = "auto";
      textareaRef.style.height = `${Math.min(textareaRef.scrollHeight, 120)}px`;
    }
  });

  return (
    <form class="composer" onSubmit={handleSend}>
      <div class="ta">
        <textarea
          ref={textareaRef}
          placeholder="Type your message..."
          value={props.prompt()}
          onInput={(e) => props.setPrompt(e.currentTarget.value)}
        />
      </div>
      <div class="foot">
        <ModelSelector
          label={props.models().find(m => m.id === props.selectedModel())?.label || "Select Model"}
          onChange={props.setSelectedModel}
          models={props.models()}
          loading={props.modelsLoading()}
        />
        <Chip label="Search" icon={<svg viewBox="0 0 12 12"><circle cx="4.5" cy="4.5" r="3.5" stroke-width="1.5" fill="none" /><path d="M7.5 7.5L10.5 10.5" stroke-width="1.5" stroke-linecap="round" /></svg>} />
        <Chip circular icon={<svg viewBox="0 0 12 12"><path d="M3 4.5a.5.5 0 0 0-.5.5v3a.5.5 0 0 0 .5.5h6a.5.5 0 0 0 .5-.5V5a.5.5 0 0 0-.5-.5h-2a.5.5 0 0 1 0-1h2A1.5 1.5 0 0 1 10.5 5v3a1.5 1.5 0 0 1-1.5 1.5h-6A1.5 1.5 0 0 1 1.5 8V5A1.5 1.5 0 0 1 3 3.5h2a.5.5 0 0 0 0-1h-2z" /></svg>} />
        <SendButton disabled={!props.prompt().trim() || props.loading()} type="submit" />
      </div>
    </form>
  );
}