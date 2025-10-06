import { Accessor } from "solid-js";
import { For } from "solid-js";

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

interface Chat {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
}

interface Props {
  chats: Accessor<Chat[]>;
  currentChatId: Accessor<string | null>;
  onSelectChat: (chatId: string) => void;
}

export default function SidebarThreads(props: Props) {
  return (
    <div style="flex: 1; overflow-y: auto; padding: 8px 0;">
      <For each={props.chats()}>
        {(chat) => (
          <div
            style={{
              padding: "8px 12px",
              cursor: "pointer",
              background: props.currentChatId() === chat.id ? "rgba(255,255,255,0.1)" : "transparent",
              "border-radius": "6px",
              margin: "2px 4px",
              color: "var(--text-1)",
              "font-size": "13px",
              overflow: "hidden",
              "text-overflow": "ellipsis",
              "white-space": "nowrap"
            }}
            onClick={() => props.onSelectChat(chat.id)}
          >
            {chat.title}
          </div>
        )}
      </For>
    </div>
  );
}