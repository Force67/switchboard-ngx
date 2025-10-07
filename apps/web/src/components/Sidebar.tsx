import { Accessor } from "solid-js";
import SidebarNewChat from "./SidebarNewChat";
import SidebarNewFolder from "./SidebarNewFolder";
import SidebarSearch from "./SidebarSearch";
import SidebarTree from "./SidebarTree";
import SidebarFooter from "./SidebarFooter";
import { sidebarState, actions } from "./sidebarStore";
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

interface Chat {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
}

interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  reasoning?: string[];
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
}

export default function Sidebar(props: Props) {
  const handleNewFolder = () => {
    actions.createFolder();
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
          actions={actions}
          chats={props.chats()}
          currentChatId={props.currentChatId()}
          onSelectChat={props.onSelectChat}
          onNewChat={props.onNewChat}
          onNewFolder={handleNewFolder}
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