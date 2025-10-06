import { For, createSignal, Show, onMount, createMemo } from "solid-js";
import type { SidebarState, Actions, ID, Chat } from "./sidebarTypes";
import FolderNode from "./FolderNode";
import ChatRow from "./ChatRow";
import CreateInline from "./CreateInline";
import ContextMenu from "./ContextMenu";
import { DragManager } from "./dnd";

interface Props {
  state: SidebarState;
  actions: Actions;
  chats: Chat[];
  currentChatId: string | null;
  onSelectChat: (chatId: string) => void;
  onNewChat: (folderId?: string) => void;
  onNewFolder: () => void;
}

export default function SidebarTree(props: Props) {
  const [contextMenu, setContextMenu] = createSignal<{ x: number; y: number } | null>(null);
  const [createInline, setCreateInline] = createSignal<{ parentId?: ID; index: number } | null>(null);
  const [dragManager] = createSignal(() => new DragManager({
    onDragStart: (kind, id, fromFolderId) => {
      // TODO: Update state.drag
      console.log("Drag start", kind, id, fromFolderId);
    },
    onDragMove: (over) => {
      // TODO: Update state.drag.over
      console.log("Drag move", over);
    },
    onDragEnd: (target) => {
      // TODO: Apply the move
      console.log("Drag end", target);
    },
    onAutoExpand: (folderId) => {
      props.actions.setCollapsed(folderId, false);
    }
  }));

  let treeRef: HTMLDivElement | undefined;

  onMount(() => {
    // TODO: Register drag handlers for all rows
  });

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  const getRootContextMenuItems = () => [
    {
      label: "New chat",
      action: () => {
        props.onNewChat();
      },
      icon: "M6 3.5A5.5 5.5 0 0 1 14.5 8h-3.673A2.18 2.18 0 0 0 6.22 6.096L4.16 8.16a.75.75 0 0 1-1.061-1.061l2.064-2.064A2.18 2.18 0 0 0 3.673 5.5H.5A5.5 5.5 0 0 1 6 3.5zM1.5 8a5.5 5.5 0 0 1 8.5-4.673V.5a.75.75 0 0 1 1.5 0v3.827A5.5 5.5 0 0 1 1.5 8zm0 0h3.673a2.18 2.18 0 0 0 1.947 1.404l2.064-2.064a.75.75 0 0 1 1.061 1.061l-2.064 2.064A2.18 2.18 0 0 0 9.827 13.5H13.5a.75.75 0 0 1 0 1.5h-3.827A5.5 5.5 0 0 1 1.5 8z"
    },
    {
      label: "New folder",
      action: () => {
        setCreateInline({ index: 0 });
      },
      icon: "M2 3.5A2.5 2.5 0 0 1 4.5 1h7A2.5 2.5 0 0 1 14 3.5v9a2.5 2.5 0 0 1-2.5 2.5h-7A2.5 2.5 0 0 1 2 12.5v-9zM4.5 2A1.5 1.5 0 0 0 3 3.5v9A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 11.5 2h-7zM8 5a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 5z"
    },
    {
      label: "Paste",
      action: () => {
        // TODO: Implement paste
        console.log("Paste");
      },
      icon: "M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1v-1z"
    }
  ];

  const handleCreateConfirm = (name: string) => {
    if (createInline()) {
      props.actions.createFolder(createInline()!.parentId);
      // TODO: Actually create with name
    }
    setCreateInline(null);
  };

  const handleCreateCancel = () => {
    setCreateInline(null);
  };

  const getOrderedFolders = () => {
    return props.state.folderOrder.map(id => props.state.folders[id]).filter(Boolean);
  };

  const getOrderedRootChats = createMemo(() => {
    // Return only chats that don't have a folderId (root level chats)
    return props.chats.filter(chat => !chat.folderId);
  });

  return (
    <div ref={treeRef} class="tree" onContextMenu={handleContextMenu}>
      <For each={getOrderedFolders()}>
        {(folder) => {
          const subfolderIds = props.state.subfolderOrder[folder.id] || [];
          const folderChats = createMemo(() => props.chats.filter(chat => chat.folderId === folder.id));
          return (
            <FolderNode
              folder={folder}
              depth={1}
              subfolders={subfolderIds
                .map(id => props.state.folders[id])
                .filter(Boolean)}
              chats={folderChats()}
              isSelected={false} // TODO: Implement selection
              currentChatId={props.currentChatId}
              onSelect={() => {
                // TODO: Update selection
              }}
              onSelectChat={props.onSelectChat}
              onNewChat={props.onNewChat}
              actions={props.actions}
              folders={props.state.folders}
              folderOrder={props.state.folderOrder}
              subfolderOrder={props.state.subfolderOrder}
              chatOrderByFolder={props.state.chatOrderByFolder}
              allChats={props.chats}
            />
          );
        }}
      </For>

      <Show when={createInline() && !createInline()!.parentId}>
        <CreateInline
          onConfirm={handleCreateConfirm}
          onCancel={handleCreateCancel}
        />
      </Show>

      {/* Root chats */}
      <For each={getOrderedRootChats()}>
        {(chat) => (
          <ChatRow
            chat={chat}
            isSelected={props.currentChatId === chat.id}
            depth={1}
            onSelect={() => props.onSelectChat(chat.id)}
            actions={props.actions}
            folders={props.state.folders}
            folderOrder={props.state.folderOrder}
            subfolderOrder={props.state.subfolderOrder}
          />
        )}
      </For>

      {contextMenu() && (
        <ContextMenu
          items={getRootContextMenuItems()}
          position={contextMenu()!}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}