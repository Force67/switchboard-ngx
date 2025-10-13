import type {
  Chat as BaseChat,
  Message as BaseMessage,
  TokenUsage as BaseTokenUsage,
} from "../types/chat";
import type { ApiChat } from "../api";

export type ID = string;

export type Chat = BaseChat;
export type Message = BaseMessage;
export type TokenUsage = BaseTokenUsage;

export type Folder = {
  id: ID;
  public_id: string;
  name: string;
  color?: string;
  parentId?: ID;            // undefined => top-level
  // derived: depth = parentId ? 2 : 1
  collapsed?: boolean;      // UI state
};

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
