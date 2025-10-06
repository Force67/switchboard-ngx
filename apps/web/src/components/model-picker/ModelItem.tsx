import { Component } from "solid-js";
import { ModelMeta } from "./models";
import CapabilityBadge from "./CapabilityBadge";

interface Props {
  model: ModelMeta;
  selectedId?: string;
  onSelect: (id: string) => void;
}

const ModelItem: Component<Props> = (props) => {
  const isSelected = () => props.selectedId === props.model.id;
  const isDisabled = () => props.model.disabled;

  const handleClick = () => {
    if (!isDisabled()) {
      props.onSelect(props.model.id);
    }
  };

  const leftIcon = () => {
    if (props.model.group === 'gpt') {
      return <svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>;
    } else {
      return <svg viewBox="0 0 24 24"><path d="M12 2l2.6 5.7 6.3.9-4.6 4.5 1.1 6.3L12 16.9 6.6 19.4l1.1-6.3L3 8.6l6.3-.9L12 2z"/></svg>;
    }
  };

  return (
    <button
      class={`row ${isDisabled() ? 'disabled' : ''}`}
      onClick={handleClick}
      role="option"
      aria-selected={isSelected()}
    >
      <div class="lefticon">
        {leftIcon()}
      </div>
      <div class="name">
        {props.model.name}
        {props.model.tier === 'pro' && <span class="diamond">🔹</span>}
        {props.model.pricing && (
          <span class="pricing">
            {props.model.pricing.input !== undefined && ` $${props.model.pricing.input.toFixed(4)}`}
            {props.model.pricing.output !== undefined && ` / $${props.model.pricing.output.toFixed(4)}`}
          </span>
        )}
      </div>
      <div class="flexfill"></div>
      <div class="badges">
        {props.model.badges.map(badge => (
          <CapabilityBadge type={badge} disabled={isDisabled()} />
        ))}
      </div>
    </button>
  );
};

export default ModelItem;