interface Props {
  onClick: () => void;
  onNewGroupChat?: () => void;
}

export default function SidebarNewChat(props: Props) {
  return (
    <div class="newchat-dropdown">
      <button class="newchat" onClick={props.onClick}>
        <svg viewBox="0 0 16 16">
          <path d="M6 3.5A5.5 5.5 0 0 1 14.5 8h-3.673A2.18 2.18 0 0 0 6.22 6.096L4.16 8.16a.75.75 0 0 1-1.061-1.061l2.064-2.064A2.18 2.18 0 0 0 3.673 5.5H.5A5.5 5.5 0 0 1 6 3.5zM1.5 8a5.5 5.5 0 0 1 8.5-4.673V.5a.75.75 0 0 1 1.5 0v3.827A5.5 5.5 0 0 1 1.5 8zm0 0h3.673a2.18 2.18 0 0 0 1.947 1.404l2.064-2.064a.75.75 0 0 1 1.061 1.061l-2.064 2.064A2.18 2.18 0 0 0 9.827 13.5H13.5a.75.75 0 0 1 0 1.5h-3.827A5.5 5.5 0 0 1 1.5 8z" />
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