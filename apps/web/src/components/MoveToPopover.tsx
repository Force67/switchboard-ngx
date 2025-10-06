import { For, createSignal, onMount, onCleanup } from "solid-js";
import type { Folder, ID } from "./sidebarTypes";

interface Props {
  folders: Record<ID, Folder>;
  folderOrder: ID[];
  subfolderOrder: Record<ID, ID[]>;
  currentFolderId?: ID;
  position: { x: number; y: number };
  onSelect: (folderId?: ID) => void;
  onClose: () => void;
}

export default function MoveToPopover(props: Props) {
  const [popoverRef, setPopoverRef] = createSignal<HTMLDivElement>();

  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const popover = popoverRef();
      if (popover && !popover.contains(e.target as Node)) {
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

  const getFolderItems = () => {
    const items: { id?: ID; name: string; depth: number }[] = [
      { name: "Root", depth: 0 }
    ];

    const addFolder = (folderId: ID, depth: number) => {
      const folder = props.folders[folderId];
      if (!folder) return;

      items.push({ id: folderId, name: folder.name, depth });

      // Add subfolders
      const subfolders = props.subfolderOrder[folderId] || [];
      subfolders.forEach(subId => addFolder(subId, depth + 1));
    };

    // Add top-level folders
    props.folderOrder.forEach(folderId => addFolder(folderId, 1));

    return items;
  };

  const handleSelect = (folderId?: ID) => {
    if (folderId !== props.currentFolderId) {
      props.onSelect(folderId);
    }
    props.onClose();
  };

  return (
    <div
      ref={setPopoverRef}
      class="cmenu"
      style={{
        left: `${props.position.x}px`,
        top: `${props.position.y}px`,
        minWidth: "180px",
      }}
    >
      <For each={getFolderItems()}>
        {(item) => (
          <div
            class="mi"
            onClick={() => handleSelect(item.id)}
            style={{
              paddingLeft: `${8 + item.depth * 12}px`,
              opacity: item.id === props.currentFolderId ? 0.5 : 1,
              cursor: item.id === props.currentFolderId ? "not-allowed" : "pointer"
            }}
          >
            {item.depth > 0 && (
              <svg viewBox="0 0 16 16" width="14" height="14" style="margin-right: 4px;">
                <path d="M2 3.5A2.5 2.5 0 0 1 4.5 1h7A2.5 2.5 0 0 1 14 3.5v9a2.5 2.5 0 0 1-2.5 2.5h-7A2.5 2.5 0 0 1 2 12.5v-9zM4.5 2A1.5 1.5 0 0 0 3 3.5v9A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 11.5 2h-7z" fill="currentColor"/>
              </svg>
            )}
            <span>{item.name}</span>
          </div>
        )}
      </For>
    </div>
  );
}