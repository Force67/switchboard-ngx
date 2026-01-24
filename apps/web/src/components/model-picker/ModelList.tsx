import { Component, For, Show } from "solid-js";
import { ModelMeta } from "./models";
import ModelItem from "./ModelItem";

interface Props {
  models: ModelMeta[];
  highlightedId?: string;
  selectedIds: string[];
  multiSelect?: boolean;
  onToggle: (id: string) => void;
  onToggleFavorite: (id: string) => void;
  isFavorite: (id: string) => boolean;
}

const ModelList: Component<Props> = (props) => {
  return (
    <div class="model-picker-list" role="listbox">
      <Show
        when={props.models.length > 0}
        fallback={
          <div class="model-picker-empty">
            <span class="model-picker-empty-text">No models found</span>
          </div>
        }
      >
        <For each={props.models}>
          {(model) => (
            <div data-model-id={model.id}>
              <ModelItem
                model={model}
                highlighted={props.highlightedId === model.id}
                selected={props.selectedIds.includes(model.id)}
                multiSelect={props.multiSelect}
                onToggle={props.onToggle}
                onToggleFavorite={props.onToggleFavorite}
                isFavorite={props.isFavorite(model.id)}
              />
            </div>
          )}
        </For>
      </Show>
    </div>
  );
};

export default ModelList;
