interface Props {
  label: string;
  onOpen: () => void;
  loading: boolean;
}

export default function ModelSelector(props: Props) {
  return (
    <div class="pill" onClick={props.onOpen}>
      {props.loading ? "Loading..." : props.label}
      <svg viewBox="0 0 8 8">
        <path d="M0 2.5L4 6.5L8 2.5H0Z" />
      </svg>
    </div>
  );
}