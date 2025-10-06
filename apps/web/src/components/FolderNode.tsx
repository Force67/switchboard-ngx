import { For, createSignal, Show } from "solid-js";
import type { Folder, Chat, Actions, ID } from "./sidebarTypes";
import ChatRow from "./ChatRow";
import ContextMenu from "./ContextMenu";
import MoveToPopover from "./MoveToPopover";

interface Props {
  folder: Folder;
  depth: 1 | 2;
  subfolders: Folder[];
  chats: Chat[];
  isSelected: boolean;
  currentChatId: string | null;
  onSelect: () => void;
  onSelectChat: (chatId: string) => void;
  onNewChat: (folderId?: string) => void;
  actions: Actions;
  folders: Record<ID, Folder>;
  folderOrder: ID[];
  subfolderOrder: Record<ID, ID[]>;
  chatOrderByFolder: Record<ID, ID[]>;
  allChats: Chat[];
}

export default function FolderNode(props: Props) {
  const [contextMenu, setContextMenu] = createSignal<{ x: number; y: number } | null>(null);
  const [movePopover, setMovePopover] = createSignal<{ x: number; y: number } | null>(null);
  const [isEditing, setIsEditing] = createSignal(false);
  const [editValue, setEditValue] = createSignal(props.folder.name);
  let rowRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;

  const isCollapsed = () => props.folder.collapsed ?? false;

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  const handleDotsClick = (e: MouseEvent) => {
    e.stopPropagation();
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    setContextMenu({ x: rect.left, y: rect.bottom + 4 });
  };

  const toggleCollapsed = (e: MouseEvent) => {
    e.stopPropagation();
    props.actions.setCollapsed(props.folder.id, !isCollapsed());
  };

  const getContextMenuItems = () => {
    const items = [];

    items.push({
      label: "New chat here",
      action: () => {
        props.onNewChat(props.folder.id);
      },
      icon: "M6 3.5A5.5 5.5 0 0 1 14.5 8h-3.673A2.18 2.18 0 0 0 6.22 6.096L4.16 8.16a.75.75 0 0 1-1.061-1.061l2.064-2.064A2.18 2.18 0 0 0 3.673 5.5H.5A5.5 5.5 0 0 1 6 3.5zM1.5 8a5.5 5.5 0 0 1 8.5-4.673V.5a.75.75 0 0 1 1.5 0v3.827A5.5 5.5 0 0 1 1.5 8zm0 0h3.673a2.18 2.18 0 0 0 1.947 1.404l2.064-2.064a.75.75 0 0 1 1.061 1.061l-2.064 2.064A2.18 2.18 0 0 0 9.827 13.5H13.5a.75.75 0 0 1 0 1.5h-3.827A5.5 5.5 0 0 1 1.5 8z"
    });

    if (props.depth === 1) {
      items.push({
        label: "New folder here",
        action: () => {
          props.actions.createFolder(props.folder.id);
        },
        icon: "M2 3.5A2.5 2.5 0 0 1 4.5 1h7A2.5 2.5 0 0 1 14 3.5v9a2.5 2.5 0 0 1-2.5 2.5h-7A2.5 2.5 0 0 1 2 12.5v-9zM4.5 2A1.5 1.5 0 0 0 3 3.5v9A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 11.5 2h-7zM8 5a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 5z"
      });
    }

    items.push({
      label: "Rename",
      action: () => {
        setIsEditing(true);
        setTimeout(() => inputRef?.focus(), 0);
      },
      icon: "M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708l-10 10a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168l10-10zM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207 11.207 2.5zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5H9v-.293l6.293-6.293zm-9.761 5.175-.106.106-1.528 3.821 3.821-1.528.106-.106A.5.5 0 0 1 5 12.5V12h-.5a.5.5 0 0 1-.5-.5V11h-.5a.5.5 0 0 1-.468-.325z"
    });

    if (props.depth === 1) {
      items.push({
        label: "Move to…",
        action: () => {
          const rect = rowRef?.getBoundingClientRect();
          if (rect) {
            setMovePopover({ x: rect.right + 8, y: rect.top });
          }
        },
        icon: "M1.5 1.5A.5.5 0 0 1 2 1h4.586a.5.5 0 0 1 .353.146l4.394 4.394a.5.5 0 0 1 .146.353V14a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a.5.5 0 0 1 .5-.5zm.5 1v12a.5.5 0 0 0 .5.5h8a.5.5 0 0 0 .5-.5V6.707A.5.5 0 0 0 10.293 6L6 1.707A.5.5 0 0 0 5.707 1H2.5a.5.5 0 0 0-.5.5z"
      });

      items.push({
        label: isCollapsed() ? "Expand all" : "Collapse all",
        action: () => {
          // TODO: Implement expand/collapse all
          console.log(isCollapsed() ? "Expand all" : "Collapse all", props.folder.id);
        },
        icon: isCollapsed() ? "M3 8a5 5 0 0 1 2.687-4.354L3.5 3.5 4.5 2.5l3 3A5 5 0 1 1 3 13.5L2 12.5A4 4 0 1 0 3 8z" : "M3 8a5 5 0 0 0 2.687 4.354L3.5 12.5 4.5 13.5l3-3A5 5 0 1 0 3 2.5L2 3.5A4 4 0 1 1 3 8z"
      });
    }

    items.push({ label: "---" });

    items.push({
      label: "Delete…",
      action: () => {
        // TODO: Implement delete with confirmation
        console.log("Delete folder", props.folder.id);
      },
      icon: "M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"
    });

    return items;
  };

  const handleEditKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      const newName = editValue().trim();
      if (newName && newName !== props.folder.name) {
        props.actions.renameFolder(props.folder.id, newName);
      }
      setIsEditing(false);
    } else if (e.key === "Escape") {
      setEditValue(props.folder.name);
      setIsEditing(false);
    }
  };

  const handleEditBlur = () => {
    setIsEditing(false);
    setEditValue(props.folder.name);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      props.onSelect();
    } else if (e.key === "ArrowRight") {
      if (isCollapsed()) {
        props.actions.setCollapsed(props.folder.id, false);
      }
    } else if (e.key === "ArrowLeft") {
      if (!isCollapsed()) {
        props.actions.setCollapsed(props.folder.id, true);
      }
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
        class={`row folder depth${props.depth} ${isCollapsed() ? "collapsed" : ""} ${props.isSelected ? "selected" : ""}`}
        style={{ "padding-left": props.depth === 1 ? "8px" : "24px" }}
        onClick={props.onSelect}
        onContextMenu={handleContextMenu}
        onKeyDown={handleKeyDown}
        tabIndex={0}
        role="treeitem"
        aria-expanded={!isCollapsed()}
        aria-selected={props.isSelected}
        data-id={props.folder.id}
      >
        <div class="caret" onClick={toggleCollapsed}>
          <svg viewBox="0 0 16 16">
            <path d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/>
          </svg>
        </div>
        <div class="icon">
          <svg viewBox="0 0 16 16">
            <path d="M2 3.5A2.5 2.5 0 0 1 4.5 1h7A2.5 2.5 0 0 1 14 3.5v9a2.5 2.5 0 0 1-2.5 2.5h-7A2.5 2.5 0 0 1 2 12.5v-9zM4.5 2A1.5 1.5 0 0 0 3 3.5v9A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 11.5 2h-7z"/>
          </svg>
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
          <div class="title">{props.folder.name}</div>
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

      <Show when={!isCollapsed()}>
        <div class="children">
          <div>
            <For each={props.subfolders}>
              {(subfolder) => (
                <FolderNode
                  folder={subfolder}
                  depth={2}
                  subfolders={[]} // Subfolders can't have subfolders per spec
                  chats={props.allChats.filter(chat => chat.folderId === subfolder.id)}
                  isSelected={false} // TODO: Implement proper selection
                  currentChatId={props.currentChatId}
                  onSelect={() => {}}
                  onSelectChat={props.onSelectChat}
                  onNewChat={props.onNewChat}
                  actions={props.actions}
                  folders={props.folders}
                  folderOrder={props.folderOrder}
                  subfolderOrder={props.subfolderOrder}
                  chatOrderByFolder={props.chatOrderByFolder}
                  allChats={props.allChats}
                />
              )}
            </For>
            <For each={props.chats}>
              {(chat) => (
                <ChatRow
                  chat={chat}
                  isSelected={props.currentChatId === chat.id}
                  depth={props.depth === 1 ? 2 : 2}
                  onSelect={() => props.onSelectChat(chat.id)}
                  actions={props.actions}
                  folders={props.folders}
                  folderOrder={props.folderOrder}
                  subfolderOrder={props.subfolderOrder}
                />
              )}
            </For>
          </div>
        </div>
      </Show>

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
          currentFolderId={props.folder.parentId}
          position={movePopover()!}
          onSelect={(folderId) => {
            props.actions.moveFolder(props.folder.id, { parentId: folderId });
          }}
          onClose={() => setMovePopover(null)}
        />
      )}
    </>
  );
}