import { createSignal } from "solid-js";
import type { SidebarState, Actions, ID, Folder, Chat } from "./sidebarTypes";

const STORAGE_KEY = "switchboard.sidebar";

const loadFromStorage = (): SidebarState => {
  if (typeof window === "undefined") {
    return getInitialState();
  }

  const json = window.localStorage.getItem(STORAGE_KEY);
  if (!json) {
    return getInitialState();
  }

  try {
    const parsed = JSON.parse(json) as SidebarState;
    // Validate and sanitize the data
    return {
      ...parsed,
      folders: Object.fromEntries(
        Object.entries(parsed.folders).filter(([, folder]) => folder && typeof folder.id === "string")
      )
    };
  } catch (error) {
    console.error("Failed to parse sidebar state", error);
    return getInitialState();
  }
};

const saveToStorage = (state: SidebarState) => {
  if (typeof window === "undefined") return;

  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (error) {
    console.error("Failed to save sidebar state", error);
  }
};

const getInitialState = (): SidebarState => ({
  folders: {},
  folderOrder: [],
  subfolderOrder: {},
  chatOrderRoot: [],
  chatOrderByFolder: {},
  selection: null,
  drag: null,
});

const [sidebarState, setSidebarState] = createSignal<SidebarState>(loadFromStorage());

// Actions implementation
const actions: Actions = {
  createFolder(parentId?: ID) {
    const id = `folder_${Date.now()}`;
    const folder: Folder = {
      id,
      name: "New folder",
      parentId,
      collapsed: false,
    };

    setSidebarState(prev => {
      const newState = {
        ...prev,
        folders: { ...prev.folders, [id]: folder },
      };

      if (parentId) {
        // Add to subfolder order
        const subOrder = [...(prev.subfolderOrder[parentId] || [])];
        subOrder.push(id);
        newState.subfolderOrder = { ...prev.subfolderOrder, [parentId]: subOrder };
      } else {
        // Add to top-level folder order
        newState.folderOrder = [...prev.folderOrder, id];
      }

      saveToStorage(newState);
      return newState;
    });
  },

  renameFolder(id: ID, name: string) {
    setSidebarState(prev => {
      const newState = {
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], name }
        }
      };
      saveToStorage(newState);
      return newState;
    });
  },

  deleteFolder(id: ID, mode: "move-up"|"delete-all") {
    setSidebarState(prev => {
      const folder = prev.folders[id];
      if (!folder) return prev;

      const newState = { ...prev };
      const children: ID[] = [];

      // Collect all descendant folders and chats
      const collectDescendants = (folderId: ID) => {
        children.push(folderId);
        const subs = prev.subfolderOrder[folderId] || [];
        subs.forEach(subId => collectDescendants(subId));
      };
      collectDescendants(id);

      if (mode === "delete-all") {
        // Remove all folders and their chats
        const newFolders = { ...prev.folders };
        children.forEach(childId => delete newFolders[childId]);

        newState.folders = newFolders;
        newState.folderOrder = prev.folderOrder.filter(fid => !children.includes(fid));

        // Remove subfolder orders
        const newSubOrders = { ...prev.subfolderOrder };
        children.forEach(childId => delete newSubOrders[childId]);
        newState.subfolderOrder = newSubOrders;

        // Remove chat orders
        const newChatOrders = { ...prev.chatOrderByFolder };
        children.forEach(childId => delete newChatOrders[childId]);
        newState.chatOrderByFolder = newChatOrders;
      } else {
        // Move contents up
        const parentId = folder.parentId;
        const subs = prev.subfolderOrder[id] || [];
        const chats = prev.chatOrderByFolder[id] || [];

        if (parentId) {
          // Move to parent folder
          const parentSubs = [...(prev.subfolderOrder[parentId] || [])];
          const insertIndex = parentSubs.indexOf(id);
          parentSubs.splice(insertIndex, 1, ...subs);
          newState.subfolderOrder = { ...prev.subfolderOrder, [parentId]: parentSubs };

          // Move chats to parent
          const parentChats = [...(prev.chatOrderByFolder[parentId] || []), ...chats];
          newState.chatOrderByFolder = { ...prev.chatOrderByFolder, [parentId]: parentChats };
        } else {
          // Move to root
          newState.folderOrder = [...prev.folderOrder.filter(fid => fid !== id), ...subs];
          newState.chatOrderRoot = [...prev.chatOrderRoot, ...chats];
        }

        // Remove the folder
        const newFolders = { ...prev.folders };
        delete newFolders[id];
        newState.folders = newFolders;

        // Remove subfolder order
        const newSubOrders = { ...prev.subfolderOrder };
        delete newSubOrders[id];
        newState.subfolderOrder = newSubOrders;

        // Remove chat order
        const newChatOrders = { ...prev.chatOrderByFolder };
        delete newChatOrders[id];
        newState.chatOrderByFolder = newChatOrders;
      }

      saveToStorage(newState);
      return newState;
    });
  },

  moveChat(id: ID, target: { folderId?: ID; index?: number }) {
    setSidebarState(prev => {
      // TODO: Implement proper chat moving logic
      // For now, just update the folderId
      console.log("Move chat", id, target);
      return prev;
    });
  },

  moveFolder(id: ID, target: { parentId?: ID; index?: number }) {
    setSidebarState(prev => {
      const folder = prev.folders[id];
      if (!folder) return prev;

      const newState = { ...prev };

      // Remove from current location
      if (folder.parentId) {
        const parentSubs = [...(prev.subfolderOrder[folder.parentId] || [])];
        const index = parentSubs.indexOf(id);
        if (index > -1) {
          parentSubs.splice(index, 1);
          newState.subfolderOrder = { ...prev.subfolderOrder, [folder.parentId]: parentSubs };
        }
      } else {
        newState.folderOrder = prev.folderOrder.filter(fid => fid !== id);
      }

      // Add to new location
      if (target.parentId) {
        const parentSubs = [...(prev.subfolderOrder[target.parentId] || [])];
        const insertIndex = target.index ?? parentSubs.length;
        parentSubs.splice(insertIndex, 0, id);
        newState.subfolderOrder = { ...prev.subfolderOrder, [target.parentId]: parentSubs };
      } else {
        const insertIndex = target.index ?? prev.folderOrder.length;
        const newOrder = [...prev.folderOrder];
        newOrder.splice(insertIndex, 0, id);
        newState.folderOrder = newOrder;
      }

      // Update folder's parentId
      newState.folders = {
        ...prev.folders,
        [id]: { ...folder, parentId: target.parentId }
      };

      saveToStorage(newState);
      return newState;
    });
  },

  setCollapsed(id: ID, collapsed: boolean) {
    setSidebarState(prev => {
      const newState = {
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], collapsed }
        }
      };
      saveToStorage(newState);
      return newState;
    });
  },

  startKeyboardDrag(ref) {
    // TODO: Implement keyboard drag
    console.log("Start keyboard drag", ref);
  },
};

export { sidebarState, actions };