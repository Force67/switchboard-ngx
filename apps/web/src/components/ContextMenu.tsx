import { For, createSignal, onMount, onCleanup } from "solid-js";

export interface MenuItem {
  label: string;
  action?: () => void;
  icon?: string;
  disabled?: boolean;
  separator?: boolean;
}

interface Props {
  items: MenuItem[];
  position: { x: number; y: number };
  onClose: () => void;
}

export default function ContextMenu(props: Props) {
  const [menuRef, setMenuRef] = createSignal<HTMLDivElement>();

  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const menu = menuRef();
      if (menu && !menu.contains(e.target as Node)) {
        props.onClose();
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        props.onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKeyDown);

    onCleanup(() => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKeyDown);
    });
  });

  const handleItemClick = (item: MenuItem) => {
    if (!item.disabled && item.action) {
      item.action();
      props.onClose();
    }
  };

  return (
    <div
      ref={setMenuRef}
      class="cmenu"
      style={{
        left: `${props.position.x}px`,
        top: `${props.position.y}px`,
      }}
    >
      <For each={props.items}>
        {(item) =>
          item.separator ? (
            <div class="sep" />
          ) : (
            <div
              class={`mi ${item.disabled ? "disabled" : ""}`}
              onClick={() => handleItemClick(item)}
              style={item.disabled ? { opacity: 0.5, cursor: "not-allowed" } : {}}
            >
              {item.icon && (
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path d={item.icon} fill="currentColor" />
                </svg>
              )}
              <span>{item.label}</span>
            </div>
          )
        }
      </For>
    </div>
  );
}
