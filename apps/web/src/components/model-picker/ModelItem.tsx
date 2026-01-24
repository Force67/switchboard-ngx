import { Component, Show } from "solid-js";
import { ModelMeta } from "./models";
import CapabilityBadge from "./CapabilityBadge";
import ProviderIcon from "./ProviderIcon";

interface Props {
  model: ModelMeta;
  highlighted?: boolean;
  selected: boolean;
  multiSelect?: boolean;
  onToggle: (id: string) => void;
  onToggleFavorite: (id: string) => void;
  isFavorite: boolean;
}

const ModelItem: Component<Props> = (props) => {
  const handleClick = () => {
    props.onToggle(props.model.id);
  };

  return (
    <div
      class={`model-row ${props.selected ? "selected" : ""} ${props.highlighted ? "focused" : ""}`}
      onClick={handleClick}
      role="option"
      aria-selected={props.selected}
      tabindex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          handleClick();
        }
      }}
    >
      {/* Provider Icon */}
      <div class="model-row-icon">
        <ProviderIcon provider={props.model.provider || "openrouter"} class="model-provider-icon" />
      </div>

      {/* Model Info */}
      <div class="model-row-content">
        <div class="model-row-header">
          <span class="model-row-name">{props.model.name}</span>
          {props.model.tier === "pro" && (
            <span class="model-tier-badge" title="Premium model">
              <svg viewBox="0 0 16 16">
                <path d="M8 1l2 4.5 5 .7-3.6 3.5.85 5-4.25-2.25L3.75 14.7l.85-5L1 6.2l5-.7L8 1z" />
              </svg>
            </span>
          )}
        </div>
        <Show when={props.model.description}>
          <span class="model-row-desc">{props.model.description}</span>
        </Show>
      </div>

      {/* Right side controls */}
      <div class="model-row-actions">
        {/* Capability badges */}
        <div class="model-row-badges">
          {props.model.badges.map((badge) => (
            <CapabilityBadge type={badge} />
          ))}
        </div>

        {/* Info button */}
        <span
          class="model-info-btn"
          onClick={(e) => {
            e.stopPropagation();
            // Could open a modal with more info
          }}
          title="Model details"
          role="button"
          tabindex={0}
        >
          <svg viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 16v-4M12 8h.01" />
          </svg>
        </span>

        {/* Favorite button */}
        <span
          class="model-favorite-btn"
          onClick={(e) => {
            e.stopPropagation();
            props.onToggleFavorite(props.model.id);
          }}
          title={props.isFavorite ? "Remove from favorites" : "Add to favorites"}
          role="button"
          tabindex={0}
        >
          <svg viewBox="0 0 24 24" class={props.isFavorite ? "filled" : ""}>
            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
          </svg>
        </span>
      </div>
    </div>
  );
};

export default ModelItem;
