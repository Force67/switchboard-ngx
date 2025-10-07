import { createSignal } from "solid-js";
import type { SidebarState, Actions, ID, Folder, Chat } from "./sidebarTypes";
import { apiService, ApiFolder, ApiChat } from "../api";

const getInitialState = (): SidebarState => ({
  folders: {},
  folderOrder: [],
  subfolderOrder: {},
  chatOrderRoot: [],
  chatOrderByFolder: {},
  selection: null,
  drag: null,
});

// Convert API folder to frontend folder
const apiFolderToFolder = (apiFolder: ApiFolder, apiFolders: ApiFolder[]): Folder => {
  let parentId: string | undefined;
  if (apiFolder.parent_id) {
    const parentFolder = apiFolders.find(f => f.id === apiFolder.parent_id);
    parentId = parentFolder?.public_id;
  }

  return {
    id: apiFolder.public_id,
    public_id: apiFolder.public_id,
    name: apiFolder.name,
    color: apiFolder.color,
    parentId,
    collapsed: apiFolder.collapsed,
  };
};

// Convert API chat to frontend chat
const apiChatToChat = (apiChat: ApiChat, apiFolders: ApiFolder[]): Chat => {
  let messages: any[] = [];
  try {
    messages = JSON.parse(apiChat.messages);
  } catch (e) {
    console.error("Failed to parse chat messages", e);
  }

  let folderId: string | undefined;
  if (apiChat.folder_id) {
    const folder = apiFolders.find(f => f.id === apiChat.folder_id);
    folderId = folder?.public_id;
  }

  return {
    id: apiChat.public_id,
    public_id: apiChat.public_id,
    title: apiChat.title,
    messages,
    createdAt: new Date(apiChat.created_at),
    folderId,
    updatedAt: apiChat.updated_at,
  };
};

const [sidebarState, setSidebarState] = createSignal<SidebarState>(getInitialState());
const [authToken, setAuthToken] = createSignal<string | null>(null);
const [isLoading, setIsLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

// Initialize data from API
const initializeFromAPI = async (token: string) => {
  setAuthToken(token);
  try {
    const [apiFolders, apiChats] = await Promise.all([
      apiService.listFolders(token),
      apiService.listChats(token),
    ]);

    // Convert API data to frontend format
    const folders: Record<string, Folder> = {};
    const folderOrder: string[] = [];
    const subfolderOrder: Record<string, string[]> = {};

    // Process folders
    for (const apiFolder of apiFolders) {
      const folder = apiFolderToFolder(apiFolder, apiFolders);
      folders[folder.id] = folder;

      if (folder.parentId) {
        // This is a subfolder
        if (!subfolderOrder[folder.parentId]) {
          subfolderOrder[folder.parentId] = [];
        }
        subfolderOrder[folder.parentId].push(folder.id);
      } else {
        // This is a top-level folder
        folderOrder.push(folder.id);
      }
    }

    // Process chats
    const chatOrderRoot: string[] = [];
    const chatOrderByFolder: Record<string, string[]> = {};

    for (const apiChat of apiChats) {
      const chat = apiChatToChat(apiChat, apiFolders);

      if (chat.folderId) {
        // Chat is in a folder
        if (!chatOrderByFolder[chat.folderId]) {
          chatOrderByFolder[chat.folderId] = [];
        }
        chatOrderByFolder[chat.folderId].push(chat.id);
      } else {
        // Chat is in root
        chatOrderRoot.push(chat.id);
      }
    }

    setSidebarState({
      folders,
      folderOrder,
      subfolderOrder,
      chatOrderRoot,
      chatOrderByFolder,
      selection: null,
      drag: null,
    });
  } catch (error) {
    console.error("Failed to initialize sidebar from API", error);
  }
};

// Actions implementation
const actions: Actions = {
  async createFolder(parentId?: ID) {
    const token = authToken();
    if (!token) return;

    setIsLoading(true);
    setError(null);

    try {
      const apiFolder = await apiService.createFolder(token, {
        name: "New folder",
        parent_id: parentId,
      });

      const folder = apiFolderToFolder(apiFolder);

      setSidebarState(prev => {
        const newState = {
          ...prev,
          folders: { ...prev.folders, [folder.id]: folder },
        };

        if (parentId) {
          // Add to subfolder order
          const subOrder = [...(prev.subfolderOrder[parentId] || [])];
          subOrder.push(folder.id);
          newState.subfolderOrder = { ...prev.subfolderOrder, [parentId]: subOrder };
        } else {
          // Add to top-level folder order
          newState.folderOrder = [...prev.folderOrder, folder.id];
        }

        return newState;
      });
    } catch (error) {
      console.error("Failed to create folder", error);
      setError("Failed to create folder. Please try again.");
    } finally {
      setIsLoading(false);
    }
  },

  async renameFolder(id: ID, name: string) {
    const token = authToken();
    if (!token) return;

    const oldName = sidebarState().folders[id]?.name;
    setIsLoading(true);
    setError(null);

    try {
      await apiService.updateFolder(token, id, { name });

      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], name },
        },
      }));
    } catch (error) {
      console.error("Failed to rename folder", error);
      setError("Failed to rename folder. Please try again.");
      // Revert the name change in UI
      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], name: oldName },
        },
      }));
    } finally {
      setIsLoading(false);
    }
  },

  async setFolderColor(id: ID, color: string) {
    const token = authToken();
    if (!token) return;

    const oldColor = sidebarState().folders[id]?.color;
    setIsLoading(true);
    setError(null);

    try {
      await apiService.updateFolder(token, id, { color });

      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], color },
        },
      }));
    } catch (error) {
      console.error("Failed to update folder color", error);
      setError("Failed to update folder color. Please try again.");
      // Revert the color change in UI
      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], color: oldColor },
        },
      }));
    } finally {
      setIsLoading(false);
    }
  },

  async deleteFolder(id: ID, mode: "move-up"|"delete-all") {
    const token = authToken();
    if (!token) return;

    setIsLoading(true);
    setError(null);

    try {
      await apiService.deleteFolder(token, id);

      // For now, just remove from local state - in a real app you'd want to refetch
      setSidebarState(prev => {
        const folder = prev.folders[id];
        if (!folder) return prev;

        const newState = { ...prev };

        if (mode === "delete-all") {
          // Remove all descendant folders and chats
          const children: ID[] = [];
          const collectDescendants = (folderId: ID) => {
            children.push(folderId);
            const subs = prev.subfolderOrder[folderId] || [];
            subs.forEach(subId => collectDescendants(subId));
          };
          collectDescendants(id);

          const newFolders = { ...prev.folders };
          children.forEach(childId => delete newFolders[childId]);
          newState.folders = newFolders;
          newState.folderOrder = prev.folderOrder.filter(fid => !children.includes(fid));

          const newSubOrders = { ...prev.subfolderOrder };
          children.forEach(childId => delete newSubOrders[childId]);
          newState.subfolderOrder = newSubOrders;

          const newChatOrders = { ...prev.chatOrderByFolder };
          children.forEach(childId => delete newChatOrders[childId]);
          newState.chatOrderByFolder = newChatOrders;
        } else {
          // Move contents up - simplified for now
          const parentId = folder.parentId;
          const subs = prev.subfolderOrder[id] || [];
          const chats = prev.chatOrderByFolder[id] || [];

          if (parentId) {
            const parentSubs = [...(prev.subfolderOrder[parentId] || [])];
            const insertIndex = parentSubs.indexOf(id);
            parentSubs.splice(insertIndex, 1, ...subs);
            newState.subfolderOrder = { ...prev.subfolderOrder, [parentId]: parentSubs };

            const parentChats = [...(prev.chatOrderByFolder[parentId] || []), ...chats];
            newState.chatOrderByFolder = { ...prev.chatOrderByFolder, [parentId]: parentChats };
          } else {
            newState.folderOrder = [...prev.folderOrder.filter(fid => fid !== id), ...subs];
            newState.chatOrderRoot = [...prev.chatOrderRoot, ...chats];
          }

          const newFolders = { ...prev.folders };
          delete newFolders[id];
          newState.folders = newFolders;

          const newSubOrders = { ...prev.subfolderOrder };
          delete newSubOrders[id];
          newState.subfolderOrder = newSubOrders;

          const newChatOrders = { ...prev.chatOrderByFolder };
          delete newChatOrders[id];
          newState.chatOrderByFolder = newChatOrders;
        }

        return newState;
      });
    } catch (error) {
      console.error("Failed to delete folder", error);
      setError("Failed to delete folder. Please try again.");
    } finally {
      setIsLoading(false);
    }
  },

  async moveChat(id: ID, target: { folderId?: ID; index?: number }) {
    const token = authToken();
    if (!token) return;

    try {
      await apiService.updateChat(token, id, {
        folder_id: target.folderId || "",
      });

      setSidebarState(prev => {
        const newState = { ...prev };

        // Remove from current location
        if (prev.chatOrderByFolder[id]) {
          // Chat is in a folder, remove it from that folder's order
          const folderId = Object.keys(prev.chatOrderByFolder).find(fid =>
            prev.chatOrderByFolder[fid].includes(id)
          );
          if (folderId) {
            newState.chatOrderByFolder = {
              ...prev.chatOrderByFolder,
              [folderId]: prev.chatOrderByFolder[folderId].filter(chatId => chatId !== id)
            };
          }
        } else {
          // Chat is in root, remove from root order
          newState.chatOrderRoot = prev.chatOrderRoot.filter(chatId => chatId !== id);
        }

        // Add to new location
        if (target.folderId) {
          // Add to folder
          const folderChats = [...(prev.chatOrderByFolder[target.folderId] || [])];
          const insertIndex = target.index ?? folderChats.length;
          folderChats.splice(insertIndex, 0, id);
          newState.chatOrderByFolder = {
            ...prev.chatOrderByFolder,
            [target.folderId]: folderChats
          };
        } else {
          // Add to root
          const insertIndex = target.index ?? prev.chatOrderRoot.length;
          const newOrder = [...prev.chatOrderRoot];
          newOrder.splice(insertIndex, 0, id);
          newState.chatOrderRoot = newOrder;
        }

        return newState;
      });
    } catch (error) {
      console.error("Failed to move chat", error);
    }
  },

  async moveFolder(id: ID, target: { parentId?: ID; index?: number }) {
    // For now, just update local state - folder moving via API would be complex
    // as it involves updating parent_id in the database
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

      return newState;
    });
  },

  async setCollapsed(id: ID, collapsed: boolean) {
    const token = authToken();
    if (!token) {
      // Update local state even without API call for immediate UI feedback
      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], collapsed }
        }
      }));
      return;
    }

    try {
      await apiService.updateFolder(token, id, { collapsed });

      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], collapsed }
        }
      }));
    } catch (error) {
      console.error("Failed to update folder collapsed state", error);
      // Revert on error
      setSidebarState(prev => ({
        ...prev,
        folders: {
          ...prev.folders,
          [id]: { ...prev.folders[id], collapsed: !collapsed }
        }
      }));
    }
  },

  startKeyboardDrag(ref) {
    // TODO: Implement keyboard drag
    console.log("Start keyboard drag", ref);
  },
};

export { sidebarState, setSidebarState, actions, initializeFromAPI, isLoading, error, setError };