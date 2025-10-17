import { Accessor } from "solid-js";

interface UserProfile {
  id: string;
  email?: string | null;
  display_name?: string | null;
}

interface SessionData {
  token: string;
  user: UserProfile;
  expires_at: string;
}

interface Props {
  session: Accessor<SessionData | null>;
  isOpen: boolean;
  onClose: () => void;
}

export default function ProfileModal(props: Props) {
  const session = props.session();
  if (!session) return null;

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };

  return (
    <div
      class="profile-modal-backdrop"
      style={{
        position: 'fixed',
        top: '0',
        left: '0',
        right: '0',
        bottom: '0',
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 2000
      }}
      onClick={handleBackdropClick}
    >
      <div
        class="profile-modal"
        style={{
          backgroundColor: 'var(--bg-color)',
          border: '1px solid var(--border-color)',
          borderRadius: '8px',
          padding: '24px',
          minWidth: '400px',
          maxWidth: '500px',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)'
        }}
      >
        <div class="profile-header" style={{
          display: 'flex',
          alignItems: 'center',
          marginBottom: '20px',
          paddingBottom: '16px',
          borderBottom: '1px solid var(--border-color)'
        }}>
          <div style={{
            width: '64px',
            height: '64px',
            borderRadius: '50%',
            backgroundColor: 'var(--accent-color)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            marginRight: '16px',
            color: 'var(--bg-color)',
            fontSize: '24px',
            fontWeight: 'bold'
          }}>
            {session.user.display_name?.charAt(0).toUpperCase() ||
             session.user.email?.charAt(0).toUpperCase() || 'U'}
          </div>
          <div>
            <h2 style={{
              margin: '0',
              fontSize: '20px',
              fontWeight: '600',
              color: 'var(--text-color)'
            }}>
              {session.user.display_name || 'User'}
            </h2>
            <p style={{
              margin: '4px 0 0 0',
              fontSize: '14px',
              color: 'var(--text-muted)'
            }}>
              {session.user.email || 'No email'}
            </p>
          </div>
        </div>

        <div class="profile-details" style={{
          display: 'flex',
          flexDirection: 'column',
          gap: '12px'
        }}>
          <div class="detail-item" style={{
            display: 'flex',
            justifyContent: 'space-between',
            padding: '8px 0',
            borderBottom: '1px solid var(--border-color)'
          }}>
            <span style={{ color: 'var(--text-muted)', fontSize: '14px' }}>User ID</span>
            <span style={{
              color: 'var(--text-color)',
              fontSize: '14px',
              fontFamily: 'monospace',
              backgroundColor: 'var(--bg-secondary)',
              padding: '2px 6px',
              borderRadius: '4px'
            }}>
              {session.user.id}
            </span>
          </div>

          <div class="detail-item" style={{
            display: 'flex',
            justifyContent: 'space-between',
            padding: '8px 0',
            borderBottom: '1px solid var(--border-color)'
          }}>
            <span style={{ color: 'var(--text-muted)', fontSize: '14px' }}>Session Expires</span>
            <span style={{ color: 'var(--text-color)', fontSize: '14px' }}>
              {formatDate(session.expires_at)}
            </span>
          </div>
        </div>

        <div class="profile-actions" style={{
          display: 'flex',
          justifyContent: 'flex-end',
          marginTop: '24px',
          paddingTop: '16px',
          borderTop: '1px solid var(--border-color)'
        }}>
          <button
            onClick={props.onClose}
            style={{
              backgroundColor: 'var(--accent-color)',
              color: 'var(--bg-color)',
              border: 'none',
              borderRadius: '6px',
              padding: '8px 16px',
              fontSize: '14px',
              fontWeight: '500',
              cursor: 'pointer',
              transition: 'opacity 0.2s'
            }}
            onMouseEnter={(e) => e.currentTarget.style.opacity = '0.8'}
            onMouseLeave={(e) => e.currentTarget.style.opacity = '1'}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}