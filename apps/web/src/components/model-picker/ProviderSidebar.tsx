import { Component, For } from "solid-js";
import { Provider } from "./models";
import ProviderIcon from "./ProviderIcon";

interface Props {
  providers: Provider[];
  activeProvider: string;
  onSelect: (providerId: string) => void;
  favoriteCount: number;
}

const ProviderSidebar: Component<Props> = (props) => {
  return (
    <nav class="provider-sidebar" role="tablist" aria-label="Filter by provider">
      <For each={props.providers}>
        {(provider) => (
          <button
            class={`provider-nav-btn ${props.activeProvider === provider.id ? 'active' : ''}`}
            onClick={() => props.onSelect(provider.id)}
            role="tab"
            aria-selected={props.activeProvider === provider.id}
            aria-controls="model-list"
            title={provider.name}
          >
            <ProviderIcon provider={provider.icon} class="provider-nav-icon" />
            {provider.id === 'favorites' && props.favoriteCount > 0 && (
              <span class="favorite-count">{props.favoriteCount}</span>
            )}
            <span class="provider-nav-indicator" />
          </button>
        )}
      </For>
    </nav>
  );
};

export default ProviderSidebar;
