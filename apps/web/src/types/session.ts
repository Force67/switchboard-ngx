export interface SessionUser {
  id: string;
  email?: string | null;
  display_name?: string | null;
  username?: string | null;
  bio?: string | null;
  avatar_url?: string | null;
}

export interface SessionData {
  token: string;
  user: SessionUser;
  expires_at: string;
}
