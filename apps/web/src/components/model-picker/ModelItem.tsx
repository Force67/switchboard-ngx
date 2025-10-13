import { Component } from "solid-js";
import { ModelMeta } from "./models";
import CapabilityBadge from "./CapabilityBadge";
import ProviderIcon from "./ProviderIcon";

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

  
  return (
    <button
      class={`row ${isDisabled() ? 'disabled' : ''}`}
      onClick={handleClick}
      role="option"
      aria-selected={isSelected()}
    >
      <div class="model-info">
        <div class="provider-icon">
          <ProviderIcon provider={props.model.provider || 'openrouter'} class="provider-icon-svg" />
        </div>
        <div class="name">
          {props.model.name}
          {props.model.tier === 'pro' && <span class="diamond">🔹</span>}
        </div>
      </div>
      <div class="flexfill"></div>
       <div class="badges">
         {props.model.badges.map(badge => (
           <CapabilityBadge type={badge} disabled={isDisabled()} />
         ))}
       </div>
       {props.model.pricing && (
         <div class="pricing-container">
           {props.model.pricing.input !== undefined && (
             <div class="pricing-item input">
               <svg class="arrow-icon" viewBox="0 0 12 12">
                 <path d="M6 2L10 6L6 10L6 7L2 7L2 5L6 5Z" fill="currentColor"/>
               </svg>
               <span class="pricing-text">${(props.model.pricing.input * 1000000).toFixed(2)}</span>
             </div>
           )}
           {props.model.pricing.output !== undefined && (
             <div class="pricing-item output">
               <svg class="arrow-icon" viewBox="0 0 12 12">
                 <path d="M6 10L2 6L6 2L6 5L10 5L10 7L6 7Z" fill="currentColor"/>
               </svg>
               <span class="pricing-text">${(props.model.pricing.output * 1000000).toFixed(2)}</span>
             </div>
           )}
           <span class="pricing-unit">/M</span>
         </div>
       )}
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