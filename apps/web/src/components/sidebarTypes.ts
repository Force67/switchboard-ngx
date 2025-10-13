import type {
  Chat as BaseChat,
  Message as BaseMessage,
  TokenUsage as BaseTokenUsage,
  Folder as BaseFolder,
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
import type { ApiChat } from "../api";

export type ID = string;

export type Chat = BaseChat;
export type Message = BaseMessage;
export type TokenUsage = BaseTokenUsage;
export type Folder = Omit<BaseFolder, 'parent_id'> & {
  parentId?: ID;            // undefined => top-level
  // derived: depth = parentId ? 2 : 1
  collapsed?: boolean;      // UI state
};

// Export all new types
export type User = User;
export type ChatMember = ChatMember;
export type ChatInvite = ChatInvite;
export type Reaction = Reaction;
export type MessageEdit = MessageEdit;
export type MessageDeletion = MessageDeletion;
export type MessageAttachment = MessageAttachment;
export type Notification = Notification;
export type Permission = Permission;
export type Session = Session;
export type UserIdentity = UserIdentity;

export type SidebarState = {
  folders: Record<ID, Folder>;
  folderOrder: ID[];        // order for top-level and then per-folder map below
  subfolderOrder: Record<ID, ID[]>;  // key: parent folder id
  chatOrderRoot: ID[];      // root chats order
  chatOrderByFolder: Record<ID, ID[]>; // key: folder id
  selection?: { kind: "chat"|"folder"; id: ID };
  drag?: DragState | null;
};

export type DragState = {
  kind: "chat"|"folder";
  id: ID;
  fromFolderId?: ID;        // undefined if from root
  // live target info updated during drag:
  over?: { type: "folder"|"chat"|"root"|"between"; id?: ID; folderId?: ID; index?: number };
};

export type Actions = {
  createFolder(parentId?: ID, name?: string): Promise<void>;
  renameFolder(id: ID, name: string): Promise<void>;
  setFolderColor(id: ID, color: string): Promise<void>;
  deleteFolder(id: ID, mode: "move-up"|"delete-all"): Promise<void>;
  moveChat(id: ID, target: { folderId?: ID; index?: number }): Promise<ApiChat | void>;
  moveFolder(id: ID, target: { parentId?: ID; index?: number }): void;
  setCollapsed(id: ID, v: boolean): Promise<void>;
  startKeyboardDrag(ref: RowRef): void;
};

export type RowRef = {
  kind: "chat"|"folder";
  id: ID;
  element: HTMLElement;
};
