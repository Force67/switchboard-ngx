import { API_BASE } from "./config";

// Re-export types from the centralized types module
export type {
  Chat,
  ChatMember,
  ChatInvite,
  Folder,
  CreateChatRequest,
  UpdateChatRequest,
  CreateFolderRequest,
  UpdateFolderRequest,
  CreateInviteRequest,
  UpdateMemberRoleRequest,
  RespondToInviteRequest,
} from "./types/chat";

// Import generated clients
import {
  listFolders as listFoldersClient,
  createFolder as createFolderClient,
  updateFolder as updateFolderClient,
  deleteFolder as deleteFolderClient,
} from "./generated/foldersClient";

import {
  listChats as listChatsClient,
  createChat as createChatClient,
  getChat as getChatClient,
  updateChat as updateChatClient,
  deleteChat as deleteChatClient,
} from "./generated/chatsClient";

import {
  listMembers as listMembersClient,
  getMember as getMemberClient,
  updateMemberRole as updateMemberRoleClient,
  removeMember as removeMemberClient,
  leaveChat as leaveChatClient,
} from "./generated/membersClient";

import {
  listInvites as listInvitesClient,
  createInvite as createInviteClient,
  listUserInvites as listUserInvitesClient,
  getInvite as getInviteClient,
  respondToInvite as respondToInviteClient,
  deleteInvite as deleteInviteClient,
} from "./generated/invitesClient";

// Import types for the class methods
import type { ChatResponse, CreateChatRequest, UpdateChatRequest } from "./generated/chats";
import type { Folder, CreateFolderRequest, UpdateFolderRequest } from "./generated/folders";
import type { MemberResponse, UpdateMemberRoleRequest } from "./generated/members";
import type { InviteResponse, CreateInviteRequest, RespondToInviteRequest } from "./generated/invites";

class ApiService {
  // =========================================================================
  // Folder API methods
  // =========================================================================

  async listFolders(token: string): Promise<Folder[]> {
    return listFoldersClient(API_BASE, token);
  }

  async createFolder(token: string, req: CreateFolderRequest): Promise<Folder> {
    return createFolderClient(API_BASE, token, req);
  }

  async updateFolder(token: string, folderId: string, req: UpdateFolderRequest): Promise<Folder> {
    return updateFolderClient(API_BASE, token, folderId, req);
  }

  async deleteFolder(token: string, folderId: string): Promise<void> {
    return deleteFolderClient(API_BASE, token, folderId);
  }

  // =========================================================================
  // Chat API methods
  // =========================================================================

  async listChats(token: string): Promise<ChatResponse[]> {
    return listChatsClient(API_BASE, token);
  }

  async createChat(token: string, req: CreateChatRequest): Promise<ChatResponse> {
    return createChatClient(API_BASE, token, req);
  }

  async getChat(token: string, chatId: string): Promise<ChatResponse> {
    return getChatClient(API_BASE, token, chatId);
  }

  async updateChat(token: string, chatId: string, req: UpdateChatRequest): Promise<ChatResponse> {
    return updateChatClient(API_BASE, token, chatId, req);
  }

  async deleteChat(token: string, chatId: string): Promise<void> {
    return deleteChatClient(API_BASE, token, chatId);
  }

  // =========================================================================
  // Member API methods
  // =========================================================================

  async listMembers(token: string, chatId: string): Promise<MemberResponse[]> {
    return listMembersClient(API_BASE, token, chatId);
  }

  async getMember(token: string, chatId: string, memberId: string): Promise<MemberResponse> {
    return getMemberClient(API_BASE, token, chatId, memberId);
  }

  async updateMemberRole(token: string, chatId: string, memberId: string, req: UpdateMemberRoleRequest): Promise<MemberResponse> {
    return updateMemberRoleClient(API_BASE, token, chatId, memberId, req);
  }

  async removeMember(token: string, chatId: string, memberId: string): Promise<void> {
    return removeMemberClient(API_BASE, token, chatId, memberId);
  }

  async leaveChat(token: string, chatId: string): Promise<void> {
    return leaveChatClient(API_BASE, token, chatId);
  }

  // =========================================================================
  // Invite API methods
  // =========================================================================

  async listInvites(token: string, chatId: string): Promise<InviteResponse[]> {
    return listInvitesClient(API_BASE, token, chatId);
  }

  async createInvite(token: string, chatId: string, req: CreateInviteRequest): Promise<InviteResponse> {
    return createInviteClient(API_BASE, token, chatId, req);
  }

  async listUserInvites(token: string): Promise<InviteResponse[]> {
    return listUserInvitesClient(API_BASE, token);
  }

  async getInvite(token: string, inviteId: string): Promise<InviteResponse> {
    return getInviteClient(API_BASE, token, inviteId);
  }

  async respondToInvite(token: string, inviteId: string, req: RespondToInviteRequest): Promise<InviteResponse> {
    return respondToInviteClient(API_BASE, token, inviteId, req);
  }

  async deleteInvite(token: string, inviteId: string): Promise<void> {
    return deleteInviteClient(API_BASE, token, inviteId);
  }

  // Convenience methods for accept/reject
  async acceptInvite(token: string, inviteId: string): Promise<InviteResponse> {
    return this.respondToInvite(token, inviteId, { action: "accept" });
  }

  async rejectInvite(token: string, inviteId: string): Promise<InviteResponse> {
    return this.respondToInvite(token, inviteId, { action: "reject" });
  }
}

export const apiService = new ApiService();
