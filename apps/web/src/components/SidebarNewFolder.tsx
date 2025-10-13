interface Props {
  onClick: () => void;
}

export default function SidebarNewFolder(props: Props) {
  return (
    <button class="newfolder" onClick={props.onClick}>
      <svg viewBox="0 0 16 16">
        <path d="M0 2a2 2 0 0 1 2-2h5.5L8 2.5H14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V2zm15 3.5H1v10.5a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V5.5z"/>
        <path d="M4 1a1 1 0 0 0-1 1v2.5H2V2a2 2 0 0 1 2-2h5.5L10 2.5H14a1 1 0 0 1 1 1v1h-1V4H9.5L8 2.5H4a1 1 0 0 0-1 1z"/>
        <path d="M8 7a.5.5 0 0 1 .5.5v2h2a.5.5 0 0 1 0 1h-2v2a.5.5 0 0 1-1 0v-2h-2a.5.5 0 0 1 0-1h2v-2A.5.5 0 0 1 8 7z"/>
      </svg>
      New folder
    </button>
  );
}