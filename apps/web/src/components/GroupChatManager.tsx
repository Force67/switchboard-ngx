import { createSignal, createEffect, For, Show } from "solid-js";
import { apiService } from "../api";

interface ChatMember {
  id: number;
  chat_id: number;
  user_id: number;
  role: string;
  joined_at: string;
}

interface ChatInvite {
  id: number;
  chat_id: number;
  inviter_id: number;
  invitee_email: string;
  status: string;
  created_at: string;
  updated_at: string;
}

interface Props {
  chatId: string;
  session: { token: string; user: { id: string } };
  onClose: () => void;
}

export default function GroupChatManager(props: Props) {
  const [members, setMembers] = createSignal<ChatMember[]>([]);
  const [invites, setInvites] = createSignal<ChatInvite[]>([]);
  const [inviteEmail, setInviteEmail] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [activeTab, setActiveTab] = createSignal<'members' | 'invites'>('members');

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [membersData, invitesData] = await Promise.all([
        apiService.listMembers(props.session.token, props.chatId),
        apiService.listInvites(props.session.token, props.chatId)
      ]);

      setMembers(membersData);
      setInvites(invitesData);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  };

  const handleInvite = async () => {
    const email = inviteEmail().trim();
    if (!email) return;

    try {
      setLoading(true);
      await apiService.createInvite(props.session.token, props.chatId, { email });
      setInviteEmail("");
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to send invite");
    } finally {
      setLoading(false);
    }
  };

  const handleAcceptInvite = async (inviteId: number) => {
    try {
      setLoading(true);
      await apiService.acceptInvite(props.session.token, inviteId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to accept invite");
    } finally {
      setLoading(false);
    }
  };

  const handleRejectInvite = async (inviteId: number) => {
    try {
      setLoading(true);
      await apiService.rejectInvite(props.session.token, inviteId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to reject invite");
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateRole = async (memberUserId: number, newRole: string) => {
    try {
      setLoading(true);
      await apiService.updateMemberRole(props.session.token, props.chatId, memberUserId, { role: newRole });
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update role");
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveMember = async (memberUserId: number) => {
    if (!confirm("Are you sure you want to remove this member?")) return;

    try {
      setLoading(true);
      await apiService.removeMember(props.session.token, props.chatId, memberUserId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to remove member");
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    loadData();
  });

  return (
    <div style={{
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: "rgba(0,0,0,0.5)",
      display: "flex",
      "align-items": "center",
      "justify-content": "center",
      "z-index": 1000
    }} onClick={props.onClose}>
      <div style={{
        background: "var(--bg-1)",
        "border-radius": "12px",
        padding: "24px",
        width: "500px",
        "max-width": "90vw",
        "max-height": "80vh",
        overflow: "auto",
        "box-shadow": "0 20px 40px rgba(0,0,0,0.3)"
      }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", "justify-content": "space-between", "align-items": "center", "margin-bottom": "20px" }}>
          <h2 style={{ margin: 0, color: "var(--text-0)" }}>Group Chat Management</h2>
          <button
            onClick={props.onClose}
            style={{
              background: "none",
              border: "none",
              color: "var(--text-1)",
              cursor: "pointer",
              "font-size": "24px",
              padding: "4px"
            }}
          >
            ×
          </button>
        </div>

        {error() && (
          <div style={{
            padding: "12px",
            "background": "rgba(239, 68, 68, 0.1)",
            "border": "1px solid rgba(239, 68, 68, 0.2)",
            "border-radius": "8px",
            color: "#ef4444",
            "margin-bottom": "16px"
          }}>
            {error()}
          </div>
        )}

        <div style={{ display: "flex", gap: "4px", "margin-bottom": "20px" }}>
          <button
            onClick={() => setActiveTab('members')}
            style={{
              padding: "8px 16px",
              "border-radius": "6px",
              border: "none",
              background: activeTab() === 'members' ? "var(--bg-3)" : "var(--bg-2)",
              color: "var(--text-0)",
              cursor: "pointer"
            }}
          >
            Members ({members().length})
          </button>
          <button
            onClick={() => setActiveTab('invites')}
            style={{
              padding: "8px 16px",
              "border-radius": "6px",
              border: "none",
              background: activeTab() === 'invites' ? "var(--bg-3)" : "var(--bg-2)",
              color: "var(--text-0)",
              cursor: "pointer"
            }}
          >
            Invites ({invites().length})
          </button>
        </div>

        <Show when={activeTab() === 'members'}>
          <div>
            <h3 style={{ margin: "0 0 16px 0", color: "var(--text-0)" }}>Members</h3>
            <For each={members()}>
              {(member) => (
                <div style={{
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "space-between",
                  padding: "12px",
                  "background": "var(--bg-2)",
                  "border-radius": "8px",
                  "margin-bottom": "8px"
                }}>
                  <div>
                    <div style={{ "font-weight": "bold", color: "var(--text-0)" }}>
                      User {member.user_id}
                      {member.user_id.toString() === props.session.user.id && " (You)"}
                    </div>
                    <div style={{ "font-size": "12px", color: "var(--text-1)" }}>
                      {member.role} • Joined {new Date(member.joined_at).toLocaleDateString()}
                    </div>
                  </div>
                  <div style={{ display: "flex", gap: "8px" }}>
                    <select
                      value={member.role}
                      onChange={(e) => handleUpdateRole(member.user_id, e.currentTarget.value)}
                      disabled={loading() || member.user_id.toString() === props.session.user.id}
                      style={{
                        padding: "4px 8px",
                        "border-radius": "4px",
                        border: "1px solid var(--bg-3)",
                        background: "var(--bg-1)",
                        color: "var(--text-0)"
                      }}
                    >
                      <option value="member">Member</option>
                      <option value="admin">Admin</option>
                      <option value="owner">Owner</option>
                    </select>
                    <Show when={member.role !== 'owner' && member.user_id.toString() !== props.session.user.id}>
                      <button
                        onClick={() => handleRemoveMember(member.user_id)}
                        disabled={loading()}
                        style={{
                          padding: "4px 8px",
                          "border-radius": "4px",
                          border: "none",
                          background: "#ef4444",
                          color: "white",
                          cursor: "pointer"
                        }}
                      >
                        Remove
                      </button>
                    </Show>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>

        <Show when={activeTab() === 'invites'}>
          <div>
            <h3 style={{ margin: "0 0 16px 0", color: "var(--text-0)" }}>Invites</h3>

            <div style={{ display: "flex", gap: "8px", "margin-bottom": "16px" }}>
              <input
                type="email"
                placeholder="Enter email address"
                value={inviteEmail()}
                onInput={(e) => setInviteEmail(e.currentTarget.value)}
                style={{
                  flex: 1,
                  padding: "8px 12px",
                  "border-radius": "6px",
                  border: "1px solid var(--bg-3)",
                  background: "var(--bg-1)",
                  color: "var(--text-0)"
                }}
              />
              <button
                onClick={handleInvite}
                disabled={loading() || !inviteEmail().trim()}
                style={{
                  padding: "8px 16px",
                  "border-radius": "6px",
                  border: "none",
                  background: "var(--bg-3)",
                  color: "var(--text-0)",
                  cursor: "pointer"
                }}
              >
                Invite
              </button>
            </div>

            <For each={invites()}>
              {(invite) => (
                <div style={{
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "space-between",
                  padding: "12px",
                  "background": "var(--bg-2)",
                  "border-radius": "8px",
                  "margin-bottom": "8px"
                }}>
                  <div>
                    <div style={{ "font-weight": "bold", color: "var(--text-0)" }}>
                      {invite.invitee_email}
                    </div>
                    <div style={{ "font-size": "12px", color: "var(--text-1)" }}>
                      {invite.status} • Sent {new Date(invite.created_at).toLocaleDateString()}
                    </div>
                  </div>
                  <Show when={invite.status === 'pending'}>
                    <div style={{ display: "flex", gap: "8px" }}>
                      <button
                        onClick={() => handleAcceptInvite(invite.id)}
                        disabled={loading()}
                        style={{
                          padding: "4px 8px",
                          "border-radius": "4px",
                          border: "none",
                          background: "#10b981",
                          color: "white",
                          cursor: "pointer"
                        }}
                      >
                        Accept
                      </button>
                      <button
                        onClick={() => handleRejectInvite(invite.id)}
                        disabled={loading()}
                        style={{
                          padding: "4px 8px",
                          "border-radius": "4px",
                          border: "none",
                          background: "#ef4444",
                          color: "white",
                          cursor: "pointer"
                        }}
                      >
                        Reject
                      </button>
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};