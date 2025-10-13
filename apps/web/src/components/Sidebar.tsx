import { Accessor } from "solid-js";
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
}

export default function Sidebar(props: Props) {
  const handleNewFolder = () => {
    props.actions.createFolder();
  };

  return (
    <div class="sidebar">
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
  );
}
