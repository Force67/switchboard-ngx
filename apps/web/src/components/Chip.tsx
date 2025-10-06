interface Props {
  label?: string;
  icon?: JSX.Element;
  circular?: boolean;
  onClick?: () => void;
}

export default function Chip(props: Props) {
  return (
    <div class={`pill ${props.circular ? "circle" : ""}`} onClick={props.onClick}>
      {props.icon}
      {props.label}
    </div>
  );
}