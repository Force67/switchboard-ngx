import { Accessor, Show } from "solid-js";
import SidebarNewChat from "./SidebarNewChat";
import SidebarNewFolder from "./SidebarNewFolder";
import SidebarSearch from "./SidebarSearch";
import SidebarTree from "./SidebarTree";
import SidebarFooter from "./SidebarFooter";
import { sidebarState } from "./sidebarStore";
import type { Chat } from "../types/chat";
import type { Actions } from "./sidebarTypes";
import "./sidebar-folders.css";

interface SessionData {
  token: string;
  user: {
    id: string;
    email?: string | null;
    display_name?: string | null;
  };
  expires_at: string;
}

interface Props {
  session: Accessor<SessionData | null>;
  chats: Accessor<Chat[]>;
  currentChatId: Accessor<string | null>;
  onLogin: () => void;
  onLogout: () => void;
  onNewChat: (folderId?: string) => void;
  onNewGroupChat?: (folderId?: string) => void;
  onSelectChat: (chatId: string) => void;
  onRenameChat: (chatId: string, title: string) => void;
  onDeleteChat: (chatId: string) => void;
  onDeleteFolder: (folderId: string) => void;
  actions: Actions;
  isOpen?: Accessor<boolean>;
  onClose?: () => void;
}

export default function Sidebar(props: Props) {
  const handleNewFolder = () => {
    props.actions.createFolder();
  };

  const isOpen = () => props.isOpen?.() ?? false;

  return (
    <>
      {/* Mobile backdrop */}
      <Show when={isOpen()}>
        <div
          class="sidebar-backdrop"
          onClick={() => props.onClose?.()}
          aria-hidden="true"
        />
      </Show>

      <div class={`sidebar ${isOpen() ? "open" : ""}`}>
        {/* Mobile close button */}
        <button
          class="sidebar-close-btn hide-desktop"
          onClick={() => props.onClose?.()}
          aria-label="Close sidebar"
        >
          <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" fill="none" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>

        <div class="sidebar-header">
           <div class="sidebar-actions">
             <SidebarNewChat onClick={props.onNewChat} onNewGroupChat={props.onNewGroupChat} />
             <SidebarNewFolder onClick={handleNewFolder} />
           </div>
          <SidebarSearch />
        </div>
        <div class="sidebar-content">
          <SidebarTree
            state={sidebarState()}
            actions={props.actions}
            chats={props.chats()}
            currentChatId={props.currentChatId()}
            onSelectChat={props.onSelectChat}
            onNewChat={props.onNewChat}
            onNewFolder={handleNewFolder}
            onRenameChat={props.onRenameChat}
            onDeleteChat={props.onDeleteChat}
            onDeleteFolder={props.onDeleteFolder}
          />
        </div>
        <SidebarFooter
          session={props.session}
          onLogin={props.onLogin}
          onLogout={props.onLogout}
        />
      </div>
    </>
  );
}
