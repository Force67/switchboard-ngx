import { createSignal, onMount } from "solid-js";
import type { Chat, Actions, ID, Folder } from "./sidebarTypes";
import ContextMenu from "./ContextMenu";
import MoveToPopover from "./MoveToPopover";

interface Props {
  chat: Chat;
  isSelected: boolean;
  depth: 1 | 2;
  onSelect: () => void;
  actions: Actions;
  folders: Record<ID, Folder>;
  folderOrder: ID[];
  subfolderOrder: Record<ID, ID[]>;
}

export default function ChatRow(props: Props) {
  const [contextMenu, setContextMenu] = createSignal<{ x: number; y: number } | null>(null);
  const [movePopover, setMovePopover] = createSignal<{ x: number; y: number } | null>(null);
  const [isEditing, setIsEditing] = createSignal(false);
  const [editValue, setEditValue] = createSignal(props.chat.title);
  let rowRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;

  onMount(() => {
    // Register with drag manager if needed
  });

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  const handleDotsClick = (e: MouseEvent) => {
    e.stopPropagation();
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    setContextMenu({ x: rect.left, y: rect.bottom + 4 });
  };

  const getContextMenuItems = () => [
    {
      label: "Open in new tab",
      action: () => {
        // TODO: Implement open in new tab
        console.log("Open in new tab", props.chat.id);
      },
      icon: "M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0zM2.04 4.326c.325 1.329 2.532 2.54 3.717 3.19.48.263.793.434.743.484-.08.08-.162.158-.242.234-.416.396-.787.749-.758 1.266.035.634.618.824 1.214 1.017.577.188 1.168.38 1.286.983.082.417-.075.988-.22 1.52-.215.782-.406 1.48.22 1.48.51 0 .759-.354.964-.713.3-.54.517-1.2.54-1.2.647.24 1.957.712 1.957.712.847 0 1.267-.634 1.267-.634.622-.363.596-.982.343-1.428-.25-.446-.491-.663-.491-.663s.265-.976.265-.976c.76-.339 1.508-.735 1.508-.735.472-.283.57-.506.57-.506s.377-.372.566-.506c.19-.135.43-.31.43-.31s.493-.176.693-.31c.2-.135.373-.306.373-.306s.378-.188.451-.377c.074-.188.074-.431 0-.431-.074-.188-.268-.334-.268-.334s-.198-.188-.397-.334c-.2-.147-.397-.334-.397-.334s-.531-.2-.73-.334c-.2-.135-.397-.334-.397-.334s-.397-.2-.531-.334c-.135-.135-.265-.2-.265-.2s-.265-.135-.397-.2c-.135-.066-.265-.135-.265-.135s-.265-.066-.397-.135c-.135-.066-.265-.135-.265-.135z"
    },
    {
      label: "Rename",
      action: () => {
        setIsEditing(true);
        setTimeout(() => inputRef?.focus(), 0);
      },
      icon: "M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708l-10 10a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168l10-10zM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207 11.207 2.5zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5H9v-.293l6.293-6.293zm-9.761 5.175-.106.106-1.528 3.821 3.821-1.528.106-.106A.5.5 0 0 1 5 12.5V12h-.5a.5.5 0 0 1-.5-.5V11h-.5a.5.5 0 0 1-.468-.325z"
    },
    {
      label: "Move to…",
      action: () => {
        const rect = rowRef?.getBoundingClientRect();
        if (rect) {
          setMovePopover({ x: rect.right + 8, y: rect.top });
        }
      },
      icon: "M1.5 1.5A.5.5 0 0 1 2 1h4.586a.5.5 0 0 1 .353.146l4.394 4.394a.5.5 0 0 1 .146.353V14a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a.5.5 0 0 1 .5-.5zm.5 1v12a.5.5 0 0 0 .5.5h8a.5.5 0 0 0 .5-.5V6.707A.5.5 0 0 0 10.293 6L6 1.707A.5.5 0 0 0 5.707 1H2.5a.5.5 0 0 0-.5.5z"
    },
    { label: "---" },
    {
      label: "Duplicate",
      action: () => {
        // TODO: Implement duplicate
        console.log("Duplicate", props.chat.id);
      },
      icon: "M4 6a2 2 0 1 1 4 0 2 2 0 0 1-4 0zm8 0a2 2 0 1 1 4 0 2 2 0 0 1-4 0zM2 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4zm8 0a2 2 0 1 0 0 4 2 2 0 0 0 0-4zm4-6a2 2 0 1 1-4 0 2 2 0 0 1 4 0zM6 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4z"
    },
    {
      label: "Delete",
      action: () => {
        // TODO: Implement delete with confirmation
        console.log("Delete", props.chat.id);
      },
      icon: "M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"
    }
  ];

  const handleEditKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      const newTitle = editValue().trim();
      if (newTitle && newTitle !== props.chat.title) {
        // TODO: Implement rename action
        console.log("Rename chat", props.chat.id, newTitle);
      }
      setIsEditing(false);
    } else if (e.key === "Escape") {
      setEditValue(props.chat.title);
      setIsEditing(false);
    }
  };

  const handleEditBlur = () => {
    setIsEditing(false);
    setEditValue(props.chat.title);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      props.onSelect();
    } else if (e.key === "F2") {
      e.preventDefault();
      setIsEditing(true);
      setTimeout(() => inputRef?.focus(), 0);
    }
  };

  return (
    <>
      <div
        ref={rowRef}
        class={`row chat ${props.isSelected ? "selected" : ""}`}
        style={{ "padding-left": props.depth === 1 ? "8px" : "24px" }}
        onClick={props.onSelect}
        onContextMenu={handleContextMenu}
        onKeyDown={handleKeyDown}
        tabIndex={0}
        role="treeitem"
        aria-selected={props.isSelected}
        data-id={props.chat.id}
        data-folder-id={props.chat.folderId || ""}
      >
        <div class="icon">
          {props.chat.isGroup ? (
            <svg viewBox="0 0 16 16">
              <path d="M7 14s-1 0-1-1 1-4 5-4 5 3 5 4-1 1-1 1H7Zm4-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm-5.784 6A2.238 2.238 0 0 1 5 13c0-1.355.68-2.75 1.936-3.72A6.325 6.325 0 0 0 5 9c-4 0-5 3-5 4s1 1 1 1h4.216ZM4.5 8a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z"/>
            </svg>
          ) : (
            <svg viewBox="0 0 16 16">
              <path d="M2 3.5A2.5 2.5 0 0 1 4.5 1h7A2.5 2.5 0 0 1 14 3.5v9a2.5 2.5 0 0 1-2.5 2.5h-7A2.5 2.5 0 0 1 2 12.5v-9zM4.5 2A1.5 1.5 0 0 0 3 3.5v9A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 11.5 2h-7z"/>
            </svg>
          )}
        </div>
        {isEditing() ? (
          <input
            ref={inputRef}
            type="text"
            value={editValue()}
            onInput={(e) => setEditValue(e.currentTarget.value)}
            onKeyDown={handleEditKeyDown}
            onBlur={handleEditBlur}
            class="title"
            style={{ border: "none", background: "transparent", outline: "none", flex: 1 }}
          />
        ) : (
          <div class="title">{props.chat.title}</div>
        )}
        <div class="end">
          <button class="dots" onClick={handleDotsClick}>
            <svg viewBox="0 0 16 16" width="12" height="12">
              <circle cx="8" cy="2" r="1.5"/>
              <circle cx="8" cy="8" r="1.5"/>
              <circle cx="8" cy="14" r="1.5"/>
            </svg>
          </button>
        </div>
      </div>

      {contextMenu() && (
        <ContextMenu
          items={getContextMenuItems()}
          position={contextMenu()!}
          onClose={() => setContextMenu(null)}
        />
      )}

      {movePopover() && (
        <MoveToPopover
          folders={props.folders}
          folderOrder={props.folderOrder}
          subfolderOrder={props.subfolderOrder}
          currentFolderId={props.chat.folderId}
          position={movePopover()!}
          onSelect={(folderId) => {
            props.actions.moveChat(props.chat.id, { folderId });
          }}
          onClose={() => setMovePopover(null)}
        />
      )}
    </>
  );
}