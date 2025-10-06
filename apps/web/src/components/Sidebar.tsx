import { Accessor } from "solid-js";
import SidebarNewChat from "./SidebarNewChat";
import SidebarSearch from "./SidebarSearch";
import SidebarThreads from "./SidebarThreads";
import SidebarFooter from "./SidebarFooter";

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
  onNewChat: () => void;
  onSelectChat: (chatId: string) => void;
}

export default function Sidebar(props: Props) {
  return (
    <div class="sidebar">
      <SidebarNewChat onClick={props.onNewChat} />
      <SidebarSearch />
      <SidebarThreads
        chats={props.chats}
        currentChatId={props.currentChatId}
        onSelectChat={props.onSelectChat}
      />
      <SidebarFooter
        session={props.session}
        onLogin={props.onLogin}
        onLogout={props.onLogout}
      />
    </div>
  );
}