export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface Message {
  id?: string;
  chat_id?: string;
  user_id?: number;
  role: "user" | "assistant" | "system";
  content: string;
  model?: string;
  usage?: TokenUsage;
  reasoning?: string[];
  timestamp?: string;
  message_type?: string;
}

export interface Chat {
  id: string;
  public_id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  folderId?: string;
  updatedAt?: string;
  isGroup?: boolean;
}
