// Re-export types for sidebar components
import type { Folder as GeneratedFolder } from "../generated/folders";
import type { LocalMessage, LocalChat, TokenUsage } from "../types/chat";

export type ID = string;

// Chat type for sidebar - uses LocalChat with parsed messages
export type Chat = LocalChat;

// Message type - use LocalMessage for streaming support
export type Message = LocalMessage;

// Re-export TokenUsage for streaming
export type { TokenUsage };

// LocalMessage for chat state with streaming fields
export type { LocalMessage };

// Folder with UI-specific properties
export type Folder = GeneratedFolder & {
  parentId?: ID;         // Alias for parent_id for UI convenience
  collapsed?: boolean;   // UI state
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
  moveChat(id: ID, target: { folderId?: ID; index?: number }): Promise<Chat | void>;
  moveFolder(id: ID, target: { parentId?: ID; index?: number }): void;
  setCollapsed(id: ID, v: boolean): Promise<void>;
  startKeyboardDrag(ref: RowRef): void;
};

export type RowRef = {
  kind: "chat"|"folder";
  id: ID;
  element: HTMLElement;
};
