interface Props {
  disabled?: boolean;
  type?: "button" | "submit";
  onClick?: () => void;
}

export default function SendButton(props: Props) {
  return (
    <button class="send" disabled={props.disabled} type={props.type || "button"} onClick={props.onClick}>
      <svg viewBox="0 0 16 16">
        <path d="M1 8.5L14 1.5L7 14.5L5.5 10.5L1 8.5Z" />
      </svg>
    </button>
  );
}