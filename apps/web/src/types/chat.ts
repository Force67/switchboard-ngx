// Re-export generated types from crudder (authoritative source)
// Chat types
export type {
  ChatResponse as Chat,
  ChatMemberResponse,
  ChatsResponse,
  CreateChatRequest,
  ListChatsQuery,
  UpdateChatRequest,
} from "../generated/chats";

// Message types
export type {
  MessageResponse as Message,
  MessageSenderResponse,
  MessageAttachmentResponse,
  MessagesResponse,
  CreateMessageRequest,
  UpdateMessageRequest,
  ListMessagesQuery,
} from "../generated/messages";

// Member types
export type {
  MemberResponse as ChatMember,
  MemberUserResponse,
  MembersResponse,
  ListMembersQuery,
  UpdateMemberRoleRequest,
} from "../generated/members";

// Invite types
export type {
  InviteResponse as ChatInvite,
  InviteInviterResponse,
  InvitesResponse,
  CreateInviteRequest,
  ListInvitesQuery,
  RespondToInviteRequest,
} from "../generated/invites";

// Folder types
export type {
  Folder,
  FoldersResponse,
  CreateFolderRequest,
  UpdateFolderRequest,
  ListFoldersQuery,
} from "../generated/folders";

// Auth types
export type {
  UserResponse as User,
  SessionResponse as Session,
  GithubLoginQuery,
  GithubLoginResponse,
  GithubCallbackRequest,
} from "../generated/auth";

// Notification types
export type {
  Notification,
  NotificationResponse,
  NotificationsResponse,
  ListNotificationsQuery,
  MarkNotificationReadRequest,
  BulkUpdateResponse,
  UnreadCountResponse,
} from "../generated/notifications";

// Attachment types
export type {
  AttachmentResponse as MessageAttachment,
  AttachmentUploaderResponse,
  AttachmentsResponse,
  CreateAttachmentRequest,
  ListAttachmentsQuery,
} from "../generated/attachments";

// Permission types
export type {
  Permission,
  PermissionResponse,
  PermissionsResponse,
  CreatePermissionRequest,
} from "../generated/permissions";

// ============================================================================
// Types NOT in crudder (used for streaming chat completion)
// ============================================================================

export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface WebSearchSource {
  title: string;
  url: string;
  snippet: string;
}

// Extended message type for local state (includes streaming fields)
export interface LocalMessage {
  id?: string;
  public_id?: string;
  chat_id?: string;
  sender_id?: string;
  role: "user" | "assistant" | "system";
  content: string;
  model?: string;
  usage?: TokenUsage;
  reasoning?: string[];
  web_search_sources?: WebSearchSource[];
  message_type?: string;
  thread_id?: string;
  reply_to?: string;
  created_at?: string;
  updated_at?: string;
  pending?: boolean;
}

// Re-export LocalChat from utils for convenience
export type { LocalChat } from "../utils/chatTransforms";
