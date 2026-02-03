/**
 * Utilities for transforming between API types (generated) and local state types.
 * Crudder generates the authoritative API types, but the frontend needs
 * some additional fields for UI state (like parsed messages, pending flags, etc.)
 */

import type { ChatResponse } from "../generated/chats";
import type { Folder } from "../generated/folders";
import type { LocalMessage, TokenUsage } from "../types/chat";

/**
 * Local chat state - extends API ChatResponse with parsed messages
 * and UI-friendly property aliases
 */
export interface LocalChat {
  // From API (using public_id as primary identifier)
  id: number;
  public_id: string;
  title: string;
  description?: string;
  avatar_url?: string;
  folder_id?: string;
  is_group: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
  member_count: number;
  message_count: number;
  last_message_at?: string;
  // Parsed messages (API stores as JSON string)
  messages: LocalMessage[];
}

/**
 * Parse messages from API JSON string to LocalMessage array
 */
export function parseMessages(messagesJson: string | undefined): LocalMessage[] {
  if (!messagesJson) return [];
  try {
    return JSON.parse(messagesJson) as LocalMessage[];
  } catch (e) {
    console.error("Failed to parse chat messages", e);
    return [];
  }
}

/**
 * Serialize LocalMessage array to JSON string for API
 */
export function serializeMessages(messages: LocalMessage[]): string {
  return JSON.stringify(messages);
}

/**
 * Transform API ChatResponse to LocalChat for frontend state
 */
export function toLocalChat(apiChat: ChatResponse): LocalChat {
  return {
    id: apiChat.id,
    public_id: apiChat.public_id,
    title: apiChat.title,
    description: apiChat.description,
    avatar_url: apiChat.avatar_url,
    folder_id: apiChat.folder_id,
    is_group: apiChat.is_group,
    created_by: apiChat.created_by,
    created_at: apiChat.created_at,
    updated_at: apiChat.updated_at,
    member_count: apiChat.member_count,
    message_count: apiChat.message_count,
    last_message_at: apiChat.last_message_at,
    messages: parseMessages(apiChat.messages),
  };
}

/**
 * Transform array of API ChatResponses to LocalChats
 */
export function toLocalChats(apiChats: ChatResponse[]): LocalChat[] {
  return apiChats.map(toLocalChat);
}

/**
 * Create a new LocalMessage for user input
 */
export function createUserMessage(content: string, userId?: number): LocalMessage {
  return {
    role: "user",
    content,
    sender_id: userId?.toString(),
    created_at: new Date().toISOString(),
  };
}

/**
 * Create a pending assistant message placeholder
 */
export function createPendingMessage(modelId: string, chatId: string): LocalMessage {
  return {
    id: `pending-${modelId}-${Date.now()}`,
    chat_id: chatId,
    role: "assistant",
    content: "",
    model: modelId,
    message_type: "text",
    created_at: new Date().toISOString(),
    pending: true,
  };
}

/**
 * Create an assistant message from streaming response
 */
export function createAssistantMessage(
  modelId: string,
  chatId: string,
  content: string,
  options?: {
    usage?: TokenUsage;
    reasoning?: string[];
    web_search_sources?: { title: string; url: string; snippet: string }[];
  }
): LocalMessage {
  return {
    id: `assistant-${modelId}-${Date.now()}`,
    chat_id: chatId,
    role: "assistant",
    content,
    model: modelId,
    created_at: new Date().toISOString(),
    usage: options?.usage,
    reasoning: options?.reasoning,
    web_search_sources: options?.web_search_sources,
  };
}

/**
 * Transform API Folder to local folder with parentId alias
 */
export interface LocalFolder extends Folder {
  parentId?: string;  // Alias for parent_id
  collapsed?: boolean; // UI state
}

export function toLocalFolder(apiFolder: Folder): LocalFolder {
  return {
    ...apiFolder,
    parentId: apiFolder.parent_id,
    collapsed: apiFolder.collapsed,
  };
}

export function toLocalFolders(apiFolders: Folder[]): LocalFolder[] {
  return apiFolders.map(toLocalFolder);
}
