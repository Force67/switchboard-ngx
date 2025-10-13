interface Props {
  onClick: () => void;
  onNewGroupChat?: () => void;
}

export default function SidebarNewChat(props: Props) {
  return (
    <div class="newchat-dropdown">
      <button class="newchat" onClick={props.onClick}>
        <svg viewBox="0 0 16 16">
          <path d="M0 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V2zm15 0a1 1 0 0 0-1-1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2z"/>
          <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
        </svg>
        New Chat
        <svg viewBox="0 0 12 12" style="margin-left: 4px;">
          <path d="M6 8L2 4h8z"/>
        </svg>
      </button>
      <div class="dropdown-menu">
        <button onClick={props.onClick}>Direct Chat</button>
        {props.onNewGroupChat && (
          <button onClick={props.onNewGroupChat}>Group Chat</button>
        )}
      </div>
    </div>
  );
}