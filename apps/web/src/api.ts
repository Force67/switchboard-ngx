import type { TokenUsage } from "./types/chat";
import type { SessionUser } from "./types/session";

const DEFAULT_API_BASE =
  typeof window !== "undefined" ? window.location.origin : "http://localhost:7070";
const API_BASE = import.meta.env.VITE_API_BASE ?? DEFAULT_API_BASE;

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
  id: number;
  public_id: string;
  user_id: number;
  folder_id: number | null;
  title: string;
  is_group: boolean;
  messages: string | null; // JSON string
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
  model?: string;
  usage?: TokenUsage;
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
  is_group?: boolean;
}

export interface UpdateChatRequest {
  title?: string;
  messages?: ChatMessage[];
  folder_id?: string;
}

export interface ChatMember {
  id: number;
  chat_id: number;
  user_id: number;
  role: string;
  joined_at: string;
}

export interface ChatInvite {
  id: number;
  chat_id: number;
  inviter_id: number;
  invitee_email: string;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface CreateInviteRequest {
  email: string;
}

export interface UpdateMemberRoleRequest {
  role: string;
}

export interface UpdateUserProfilePayload {
  username?: string | null;
  display_name?: string | null;
  bio?: string | null;
  avatar_url?: string | null;
}

class ApiService {
  private getAuthHeaders(token: string) {
    return {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    };
  }

  async getCurrentUser(token: string): Promise<SessionUser> {
    const response = await fetch(`${API_BASE}/api/users/me`, {
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      const body = (await response.json().catch(() => null)) as
        | { error?: string }
        | null;
      throw new Error(body?.error ?? response.statusText);
    }

    const data = await response.json();
    return data.user as SessionUser;
  }

  async updateCurrentUser(
    token: string,
    req: UpdateUserProfilePayload,
  ): Promise<SessionUser> {
    const response = await fetch(`${API_BASE}/api/users/me`, {
      method: "PATCH",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      const body = (await response.json().catch(() => null)) as
        | { error?: string }
        | null;
      throw new Error(body?.error ?? response.statusText);
    }

    const data = await response.json();
    return data.user as SessionUser;
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
    console.log('Original request:', req);

    // Remove folder_id if it's null/undefined to avoid serialization issues
    const cleanReq = { ...req };
    if (!cleanReq.folder_id) {
      delete cleanReq.folder_id;
    }

    const requestBody = JSON.stringify(cleanReq);
    console.log('Cleaned request:', cleanReq);
    console.log('Request body being sent:', requestBody);
    console.log('Using token:', token.substring(0, 20) + '...');

    const response = await fetch(`${API_BASE}/api/chats`, {
      method: "POST",
      headers: this.getAuthHeaders(token),
      body: requestBody,
    });

    console.log('Response status:', response.status, response.statusText);

    if (!response.ok) {
      const errorText = await response.text();
      console.error('Error response body:', errorText);
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

  // Member API methods
  async listMembers(token: string, chatId: string): Promise<ChatMember[]> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}/members`, {
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch members: ${response.statusText}`);
    }

    const data = await response.json();
    return data.members;
  }

  async updateMemberRole(token: string, chatId: string, memberUserId: number, req: UpdateMemberRoleRequest): Promise<ChatMember> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}/members/${memberUserId}`, {
      method: "PUT",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to update member role: ${response.statusText}`);
    }

    const data = await response.json();
    return data.member;
  }

  async removeMember(token: string, chatId: string, memberUserId: number): Promise<void> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}/members/${memberUserId}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to remove member: ${response.statusText}`);
    }
  }

  // Invite API methods
  async listInvites(token: string, chatId: string): Promise<ChatInvite[]> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}/invites`, {
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch invites: ${response.statusText}`);
    }

    const data = await response.json();
    return data.invites;
  }

  async createInvite(token: string, chatId: string, req: CreateInviteRequest): Promise<ChatInvite> {
    const response = await fetch(`${API_BASE}/api/chats/${chatId}/invites`, {
      method: "POST",
      headers: this.getAuthHeaders(token),
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to create invite: ${response.statusText}`);
    }

    const data = await response.json();
    return data.invite;
  }

  async acceptInvite(token: string, inviteId: number): Promise<void> {
    const response = await fetch(`${API_BASE}/api/invites/${inviteId}/accept`, {
      method: "POST",
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to accept invite: ${response.statusText}`);
    }
  }

  async rejectInvite(token: string, inviteId: number): Promise<void> {
    const response = await fetch(`${API_BASE}/api/invites/${inviteId}/reject`, {
      method: "POST",
      headers: this.getAuthHeaders(token),
    });

    if (!response.ok) {
      throw new Error(`Failed to reject invite: ${response.statusText}`);
    }
  }
}

export const apiService = new ApiService();
