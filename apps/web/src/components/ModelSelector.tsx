import { createSignal } from "solid-js";

interface ModelOption {
  id: string;
  label: string;
  description?: string | null;
}

interface Props {
  label: string;
  onChange: (modelId: string) => void;
  models: ModelOption[];
  loading: boolean;
}

export default function ModelSelector(props: Props) {
  const [open, setOpen] = createSignal(false);

  const handleSelect = (modelId: string) => {
    props.onChange(modelId);
    setOpen(false);
  };

  return (
    <div class="pill" onClick={() => setOpen(!open())}>
      {props.loading ? "Loading..." : props.label}
      <svg viewBox="0 0 8 8">
        <path d="M0 2.5L4 6.5L8 2.5H0Z" />
      </svg>
      {open() && (
        <div style="position: absolute; top: 100%; left: 0; background: var(--bg-2); border: 1px solid var(--hair); border-radius: 8px; z-index: 10; min-width: 200px;">
          {props.models.map(model => (
            <div
              style="padding: 8px 12px; cursor: pointer; hover: background: var(--bg-3);"
              onClick={() => handleSelect(model.id)}
            >
              {model.label}
              {model.description && <small style="color: var(--text-1);"> — {model.description}</small>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}