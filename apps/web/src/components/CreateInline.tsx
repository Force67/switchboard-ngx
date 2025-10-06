import { createSignal, onMount } from "solid-js";

interface Props {
  initialValue?: string;
  onConfirm: (value: string) => void;
  onCancel: () => void;
}

export default function CreateInline(props: Props) {
  const [value, setValue] = createSignal(props.initialValue || "New folder");
  let inputRef: HTMLInputElement | undefined;

  onMount(() => {
    inputRef?.focus();
    inputRef?.select();
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      const trimmed = value().trim();
      if (trimmed) {
        props.onConfirm(trimmed);
      } else {
        props.onCancel();
      }
    } else if (e.key === "Escape") {
      props.onCancel();
    }
  };

  const handleBlur = () => {
    const trimmed = value().trim();
    if (trimmed) {
      props.onConfirm(trimmed);
    } else {
      props.onCancel();
    }
  };

  return (
    <div class="create-inline">
      <svg viewBox="0 0 16 16" width="14" height="14">
        <path d="M2 3.5A2.5 2.5 0 0 1 4.5 1h7A2.5 2.5 0 0 1 14 3.5v9a2.5 2.5 0 0 1-2.5 2.5h-7A2.5 2.5 0 0 1 2 12.5v-9zM4.5 2A1.5 1.5 0 0 0 3 3.5v9A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 11.5 2h-7z"/>
        <path d="M8 5a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 5z"/>
      </svg>
      <input
        ref={inputRef}
        type="text"
        value={value()}
        onInput={(e) => setValue(e.currentTarget.value)}
        onKeyDown={handleKeyDown}
        onBlur={handleBlur}
        placeholder="Folder name"
      />
    </div>
  );
}