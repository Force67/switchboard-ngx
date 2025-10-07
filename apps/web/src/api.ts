const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:7070";

export interface ApiFolder {
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

export interface ApiChat {
  id: string;
  public_id: string;
  user_id: number;
  folder_id?: number;
  title: string;
  messages: string; // JSON string
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  model?: string;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  reasoning?: string[];
}

export interface CreateFolderRequest {
  name: string;
  color?: string;
  parent_id?: string;
}

export interface UpdateFolderRequest {
  name?: string;
  color?: string;
  collapsed?: boolean;
}

export interface CreateChatRequest {
  title: string;
  messages: ChatMessage[];
  folder_id?: string;
}

export interface UpdateChatRequest {
  title?: string;
  messages?: ChatMessage[];
  folder_id?: string;
}

class ApiService {
  private getAuthHeaders(token: string) {
    return {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    };
  }

  // Folder API methods
  async listFolders(token: string): Promise<ApiFolder[]> {
    const response = await fetch(`${API_BASE}/api/folders`, {
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch folders: ${response.statusText}`);
    }

    const data = await response.json();
    return data.folders;
  }

  async createFolder(token: string, req: CreateFolderRequest): Promise<ApiFolder> {
    const response = await fetch(`${API_BASE}/api/folders`, {
      method: "POST",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to create folder: ${response.statusText}`);
    }

    const data = await response.json();
    return data.folder;
  }

  async updateFolder(token: string, folderId: string, req: UpdateFolderRequest): Promise<ApiFolder> {
    const response = await fetch(`${API_BASE}/api/folders/${folderId}`, {
      method: "PUT",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to update folder: ${response.statusText}`);
    }

    const data = await response.json();
    return data.folder;
  }

  async deleteFolder(token: string, folderId: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/folders/${folderId}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to delete folder: ${response.statusText}`);
    }
  }

  // Chat API methods
  async listChats(token: string): Promise<ApiChat[]> {
    const response = await fetch(`${API_BASE}/api/chats`, {
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch chats: ${response.statusText}`);
    }

    const data = await response.json();
    return data.chats;
  }

  async createChat(token: string, req: CreateChatRequest): Promise<ApiChat> {
    const response = await fetch(`${API_BASE}/api/chats`, {
      method: "POST",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to create chat: ${response.statusText}`);
    }

    const data = await response.json();
    return data.chat;
  }

  async updateChat(token: string, chatId: string, req: UpdateChatRequest): Promise<ApiChat> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}`, {
      method: "PUT",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to update chat: ${response.statusText}`);
    }

    const data = await response.json();
    return data.chat;
  }

  async deleteChat(token: string, chatId: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to delete chat: ${response.statusText}`);
    }
  }
}

export const apiService = new ApiService();