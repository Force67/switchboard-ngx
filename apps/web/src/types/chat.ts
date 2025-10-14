export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface Message {
  id?: string;
  public_id?: string;
  chat_id?: string;
  user_id?: number;
  role: "user" | "assistant" | "system";
  content: string;
  model?: string;
  usage?: TokenUsage;
  reasoning?: string[];
  message_type?: "text" | "system" | "file";
  thread_id?: string;
  reply_to_id?: string;
  created_at?: string;
  updated_at?: string;
  pending?: boolean;
}

export interface Chat {
  id: string;
  public_id: string;
  title: string;
  chat_type: "direct" | "group" | "system";
  messages?: Message[];
  created_at: string;
  updated_at: string;
  folderId?: string;
}

export interface User {
  id: number;
  public_id: string;
  email?: string;
  display_name?: string;
  created_at: string;
  updated_at: string;
}

export interface Folder {
  id: number;
  public_id: string;
  user_id: number;
  name: string;
  color?: string;
  parent_id?: number;
  collapsed: boolean;
  created_at: string;
  updated_at: string;
}

export interface ChatMember {
  id: number;
  chat_id: number;
  user_id: number;
  role: "owner" | "admin" | "member";
  joined_at: string;
}

export interface ChatInvite {
  id: number;
  public_id: string;
  chat_id: number;
  invited_by_user_id: number;
  invited_user_id?: number;
  invited_email?: string;
  status: "pending" | "accepted" | "declined" | "expired";
  expires_at: string;
  created_at: string;
}

export interface Reaction {
  id: number;
  message_id: number;
  user_id: number;
  emoji: string;
  created_at: string;
}

// New audit log interfaces
export interface MessageEdit {
  id: number;
  message_id: number;
  edited_by_user_id: number;
  old_content: string;
  new_content: string;
  edited_at: string;
}

export interface MessageDeletion {
  id: number;
  message_id: number;
  deleted_by_user_id: number;
  reason?: string;
  deleted_at: string;
}

// File attachment interface
export interface MessageAttachment {
  id: number;
  message_id: number;
  file_name: string;
  file_type: string;
  file_url: string;
  file_size_bytes: number;
  created_at: string;
}

// Notification interface
export interface Notification {
  id: number;
  user_id: number;
  type: string;
  title: string;
  body: string;
  read: boolean;
  created_at: string;
}

// Permissions interface
export interface Permission {
  id: number;
  user_id: number;
  resource_type: string;
  resource_id: number;
  permission_level: "read" | "write" | "admin";
  granted_at: string;
}

// Session interface
export interface Session {
  id: number;
  user_id: number;
  token: string;
  created_at: string;
  expires_at: string;
}

// User identity interface for OAuth providers
export interface UserIdentity {
  id: number;
  user_id: number;
  provider: string;
  provider_uid: string;
  credential_encrypted?: string;
  created_at: string;
  updated_at: string;
}
