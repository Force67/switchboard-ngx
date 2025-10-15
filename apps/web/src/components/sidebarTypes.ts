import type {
  Chat as BaseChat,
  Message as BaseMessage,
  TokenUsage as BaseTokenUsage,
} from "../types/chat";
import type { ApiChat } from "../api";

export type {
  User,
  ChatMember,
  ChatInvite,
  Reaction,
  MessageEdit,
  MessageDeletion,
  MessageAttachment,
  Notification,
  Permission,
  Session,
  UserIdentity,
} from "../types/chat";

export type ID = string;

export type Chat = BaseChat;
export type Message = BaseMessage;
export type TokenUsage = BaseTokenUsage;

export interface Folder {
  id: ID;
  public_id: string;
  name: string;
  color?: string;
  parentId?: ID;
  collapsed?: boolean;
}

export interface SidebarState {
  folders: Record<ID, Folder>;
  folderOrder: ID[];
  subfolderOrder: Record<ID, ID[]>;
  chatOrderRoot: ID[];
  chatOrderByFolder: Record<ID, ID[]>;
  selection?: { kind: "chat" | "folder"; id: ID } | null;
  drag?: DragState | null;
}

export interface DragState {
  kind: "chat" | "folder";
  id: ID;
  fromFolderId?: ID;
  over?: { type: "folder" | "chat" | "root" | "between"; id?: ID; folderId?: ID; index?: number };
}

export interface Actions {
  createFolder(parentId?: ID, name?: string): Promise<void>;
  renameFolder(id: ID, name: string): Promise<void>;
  setFolderColor(id: ID, color: string): Promise<void>;
  deleteFolder(id: ID, mode: "move-up" | "delete-all"): Promise<void>;
  moveChat(id: ID, target: { folderId?: ID; index?: number }): Promise<ApiChat | void>;
  moveFolder(id: ID, target: { parentId?: ID; index?: number }): void;
  setCollapsed(id: ID, value: boolean): Promise<void>;
  startKeyboardDrag(ref: RowRef): void;
}

export interface RowRef {
  kind: "chat" | "folder";
  id: ID;
  element: HTMLElement;
}
