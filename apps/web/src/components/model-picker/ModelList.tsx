import { Component } from "solid-js";
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
  expanded: boolean;
}

const ModelList: Component<Props> = (props) => {
  const displayedModels = () => props.expanded ? props.models : props.models.slice(0, 8);

  return (
    <div class="list">
      {displayedModels().map(model => (
        <ModelItem
          model={model}
          highlighted={props.highlightedId === model.id}
          selected={props.selectedIds.includes(model.id)}
          multiSelect={props.multiSelect}
          onToggle={props.onToggle}
          onToggleFavorite={props.onToggleFavorite}
          isFavorite={props.isFavorite(model.id)}
        />
      ))}
    </div>
  );
};

export default ModelList;
