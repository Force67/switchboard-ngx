import { Component } from "solid-js";
import { ModelMeta } from "./models";
import CapabilityBadge from "./CapabilityBadge";

interface Props {
  model: ModelMeta;
  highlightedId?: string;
  onSelect: (id: string) => void;
  onToggleFavorite: (id: string) => void;
  isFavorite: boolean;
}

const ModelItem: Component<Props> = (props) => {
  const isSelected = () => props.highlightedId === props.model.id;
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
             {props.model.pricing.input !== undefined && ` $${(props.model.pricing.input * 1000000).toFixed(2)}/M`}
             {props.model.pricing.output !== undefined && ` / $${(props.model.pricing.output * 1000000).toFixed(2)}/M`}
           </span>
         )}
      </div>
      <div class="flexfill"></div>
       <div class="badges">
         {props.model.badges.map(badge => (
           <CapabilityBadge type={badge} disabled={isDisabled()} />
         ))}
       </div>
       <span
         class="favorite-btn"
         onClick={(e) => {
           e.stopPropagation();
           props.onToggleFavorite(props.model.id);
         }}
         title={props.isFavorite ? "Remove from favorites" : "Add to favorites"}
       >
         <svg viewBox="0 0 24 24" class={props.isFavorite ? 'filled' : ''}>
           <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
         </svg>
       </span>
    </button>
  );
};

export default ModelItem;