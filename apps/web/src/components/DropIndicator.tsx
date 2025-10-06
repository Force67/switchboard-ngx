interface Props {
  type: "line" | "into";
  rect?: DOMRect;
}

export default function DropIndicator(props: Props) {
  if (!props.rect) return null;

  const style = {
    position: "fixed" as const,
    zIndex: 999,
    pointerEvents: "none" as const,
    left: props.type === "line" ? `${props.rect.left + 8}px` : `${props.rect.left}px`,
    top: props.type === "line" ? `${props.rect.top - 1}px` : `${props.rect.top}px`,
    width: props.type === "line" ? `${props.rect.width - 16}px` : `${props.rect.width}px`,
    height: props.type === "line" ? "2px" : `${props.rect.height}px`,
  };

  return (
    <div
      class={props.type === "line" ? "drop-line" : "drop-into"}
      style={style}
    />
  );
}