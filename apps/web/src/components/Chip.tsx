interface Props {
  label?: string;
  icon?: JSX.Element;
  circular?: boolean;
}

export default function Chip(props: Props) {
  return (
    <div class={`pill ${props.circular ? "circle" : ""}`}>
      {props.icon}
      {props.label}
    </div>
  );
}