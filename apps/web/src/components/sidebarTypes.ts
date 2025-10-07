export type ID = string;

export type Folder = {
  id: ID;
  public_id: string;
  name: string;
  color?: string;
  parentId?: ID;            // undefined => top-level
  // derived: depth = parentId ? 2 : 1
  collapsed?: boolean;      // UI state
};

export type Chat = {
  id: string;
  public_id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  folderId?: ID;            // undefined => root
  updatedAt?: string;
};

export type Message = {
  role: "user" | "assistant";
  content: string;
  model?: string;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  reasoning?: string[];
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
  createFolder(parentId?: ID, name?: string): void;
  renameFolder(id: ID, name: string): void;
  setFolderColor(id: ID, color: string): void;
  deleteFolder(id: ID, mode: "move-up"|"delete-all"): void;
  moveChat(id: ID, target: { folderId?: ID; index?: number }): void;
  moveFolder(id: ID, target: { parentId?: ID; index?: number }): void;
  setCollapsed(id: ID, v: boolean): void;
  startKeyboardDrag(ref: RowRef): void;
};

export type RowRef = {
  kind: "chat"|"folder";
  id: ID;
  element: HTMLElement;
};