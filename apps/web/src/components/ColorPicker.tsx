import { createSignal, onMount, onCleanup } from "solid-js";

interface Props {
  value?: string;
  onChange: (color: string) => void;
  onClose: () => void;
}

const PRESET_COLORS = [
  "#e54cbf", // Pink
  "#d7c4e6", // Purple
  "#a8b5d1", // Blue
  "#7ec4cf", // Teal
  "#6bbf7a", // Green
  "#f4d35e", // Yellow
  "#f4a261", // Orange
  "#e76f51", // Red
  "#9e9e9e", // Gray
];

export default function ColorPicker(props: Props) {
  const [selectedColor, setSelectedColor] = createSignal(props.value || PRESET_COLORS[0]);
  let containerRef: HTMLDivElement | undefined;

  const handleColorSelect = (color: string) => {
    setSelectedColor(color);
    props.onChange(color);
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      props.onClose();
    }
  };

  onMount(() => {
    document.addEventListener("mousedown", handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener("mousedown", handleClickOutside);
  });

  return (
    <div
      ref={containerRef}
      class="color-picker"
      style={{
        position: "absolute",
        top: "100%",
        left: "0",
        "z-index": "1000",
        background: "var(--bg-2)",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        "border-radius": "8px",
        padding: "8px",
        display: "grid",
        "grid-template-columns": "repeat(3, 1fr)",
        gap: "6px",
        "box-shadow": "0 8px 24px rgba(0, 0, 0, 0.5)",
      }}
    >
      {PRESET_COLORS.map(color => (
        <button
          type="button"
          class="color-option"
          style={{
            width: "24px",
            height: "24px",
            "border-radius": "4px",
            border: selectedColor() === color ? "2px solid var(--text-0)" : "2px solid transparent",
            background: color,
            cursor: "pointer",
            transition: "all 0.15s ease",
          }}
          onClick={() => handleColorSelect(color)}
          title={color}
        />
      ))}
    </div>
  );
}